use log::info;
use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};

use super::http::HttpRequest;

#[derive(Hash, PartialEq, Eq)]
enum StaticFile {
    Index,
    NotFound,
}

pub struct HttpServer {
    listener: TcpListener,
    static_files: HashMap<StaticFile, String>,
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
        let content = self.static_files.get(&StaticFile::Index).unwrap();
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
        let content = self.static_files.get(&StaticFile::NotFound).unwrap();
        let content_length = content.len();

        format!(
            "HTTP/1.1 404 Not Found\r\nContent-Length:{}\r\n\r\n{}",
            content_length, content
        )
    }
}

fn load_static_files() -> HashMap<StaticFile, String> {
    let mut map = HashMap::new();

    let index = std::fs::read_to_string("static/index.html").unwrap();
    map.insert(StaticFile::Index, index);

    let not_found = std::fs::read_to_string("static/404.html").unwrap();
    map.insert(StaticFile::NotFound, not_found);

    map
}
