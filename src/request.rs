use std::collections::HashMap;
use std::fmt::Display;
use std::io::Read;
use std::net::TcpStream;

use log::debug;
use urlencoding::decode;

use crate::state::{State, StaticFiles};

pub struct HttpRequest<T: State> {
    pub method: HttpMethod,
    pub path: Option<String>,
    pub content_type: Option<ContentType>,
    pub form_data: HashMap<String, String>,
    pub static_files: StaticFiles,
    pub state: T,
}

impl<T: State> HttpRequest<T> {
    pub fn new(tcp_stream: &TcpStream, static_files: StaticFiles, state: T) -> HttpRequest<T> {
        let request = read_raw_request(tcp_stream);
        let method = parse_method(&request);
        let path = parse_path(&request);
        let content_type = parse_content_type(&request);
        let form_data = match content_type {
            Some(ContentType::ApplicationFormUrlEncoded) => parse_form_url_encoded_params(&request),
            _ => HashMap::new(),
        };

        HttpRequest {
            method,
            path,
            content_type,
            form_data,
            static_files,
            state,
        }
    }

    pub fn route(&self) -> String {
        format!(
            "{} {}",
            self.method,
            self.path.clone().unwrap_or(String::from("-"))
        )
    }
}

fn read_raw_request(mut stream: &TcpStream) -> String {
    // Hard-coded request size is good enough for demo purposes.
    // Parsing will fail if request size is larger than buffer.
    let mut buffer = [0; 4096];

    // Read the request header
    let bytes_read = stream.read(&mut buffer).unwrap();

    if bytes_read == buffer.len() {
        panic!("Buffer overflow. Try increasing buffer size.");
    }

    // Append the header to the request
    let request = String::from_utf8_lossy(&buffer)
        .trim_matches(char::from(0)) // Remove null characters caused by large buffer.
        .to_string();

    debug!("{request}");

    request
}

fn parse_method(header: &str) -> HttpMethod {
    match header.lines().next() {
        Some(line) => line
            .split(' ')
            .next()
            .map(ToString::to_string)
            .map(HttpMethod::from)
            .unwrap_or(HttpMethod::None),
        None => HttpMethod::None,
    }
}

fn parse_path(header: &str) -> Option<String> {
    header
        .lines()
        .next()
        .map(|line| line.split(' ').nth(1))
        .flatten()
        .map(ToString::to_string)
}

fn parse_content_type(header: &str) -> Option<ContentType> {
    header_value(header, "Content-Type").map(|value| ContentType::from(value))
}

fn parse_form_url_encoded_params(header: &str) -> HashMap<String, String> {
    let parts = header.split("\r\n\r\n");
    let encoded_params = parts.last().unwrap();

    encoded_params
        .split("&")
        .map(|pair| {
            let kv: Vec<_> = pair.split('=').collect();

            if kv.len() != 2 {
                debug!("Request: {}", header);
                ("".to_string(), "".to_string())
            } else {
                let key = kv[0].to_string();
                let value = decode(kv[1]).expect("UTF-8").to_string().replace("+", " ");
                (key, value)
            }
        })
        .collect()
}

fn header_value(request_header: &str, key: &str) -> Option<String> {
    let pattern = format!("\r\n{}: ", key.to_lowercase());

    match request_header.to_lowercase().find(pattern.as_str()) {
        Some(index) => {
            let start = index + pattern.len();
            let end = request_header[start..].find('\r').unwrap() + start;
            let p = Some(request_header[start..end].to_string());
            p
        }
        None => None,
    }
}

#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Post,
    None,
}

impl From<String> for HttpMethod {
    fn from(value: String) -> Self {
        match value.as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            _ => HttpMethod::None,
        }
    }
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_uppercase())
    }
}

pub enum ContentType {
    ApplicationFormUrlEncoded,
    Html,
    Json,
    TextPlain,
}

impl From<String> for ContentType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "application/x-www-form-urlencoded" => ContentType::ApplicationFormUrlEncoded,
            other => panic!("Invalid content type {}", other),
        }
    }
}
