use log::info;
use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};

use super::http::HttpRequest;

pub struct HttpServer {
    listener: TcpListener,
    static_files: HashMap<String, String>,
    messages: HashMap<String, String>,
}

impl HttpServer {
    pub fn bind(host: &str, port: usize) -> Self {
        let address = format!("{}:{}", host, port);
        let listener = TcpListener::bind(address.as_str()).unwrap();
        let static_files = load_static_files();

        HttpServer {
            listener,
            static_files,
            messages: HashMap::new(),
        }
    }

    pub fn start(&self) {
        info!(
            "Listening at {} for connections",
            self.listener.local_addr().unwrap()
        );
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            self.handle_request(stream);
        }
    }

    fn handle_request(&self, mut stream: TcpStream) {
        let request = HttpRequest::from(&stream);
        let route = request.route();

        let response = match route.as_str() {
            "GET /" => self.handle_get_root(&request),
            "POST /users" => self.handle_post_users(&request),
            _ => self.handle_not_found(),
        };

        stream.write_all(response.as_bytes()).unwrap();
    }

    fn handle_get_root(&self, _request: &HttpRequest) -> String {
        let content = self.static_files.get("index.html").unwrap();
        let content_length = content.len();

        format!(
            "HTTP/1.1 200 OK\r\nContent-Length:{}\r\n\r\n{}",
            content_length, content
        )
    }

    fn handle_post_users(&self, request: &HttpRequest) -> String {
        let content = request.query_params.get("username").unwrap();
        let content_length = content.len();

        format!(
            "HTTP/1.1 200 OK\r\nContent-Length:{}\r\n\r\n{}",
            content_length, content
        )
    }

    fn handle_not_found(&self) -> String {
        let content = self.static_files.get("404.html").unwrap();
        let content_length = content.len();

        format!(
            "HTTP/1.1 404 Not Found\r\nContent-Length:{}\r\n\r\n{}",
            content_length, content
        )
    }
}

fn load_static_files() -> HashMap<String, String> {
    let mut map = HashMap::new();

    let paths = std::fs::read_dir("static/").unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let index = std::fs::read_to_string(&path).unwrap();
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        map.insert(filename, index);
    }

    map
}
