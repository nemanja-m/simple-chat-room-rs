use log::info;
use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};

use crate::http::HttpRequest;
use crate::room::ChatRoom;

pub struct HttpServer {
    chat_room: ChatRoom,
    static_files: HashMap<String, String>,
}

impl HttpServer {
    pub fn build() -> Self {
        let chat_room = ChatRoom::new();
        let static_files = load_static_files();

        HttpServer {
            chat_room,
            static_files,
        }
    }

    pub fn start(&mut self, host: &str, port: usize) {
        let address = format!("{}:{}", host, port);
        let listener = TcpListener::bind(address.as_str()).unwrap();

        info!(
            "Listening at {} for connections",
            listener.local_addr().unwrap()
        );

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            self.handle_request(stream);
        }
    }

    fn handle_request(&mut self, mut stream: TcpStream) {
        let request = HttpRequest::from(&stream);
        let route = request.route();

        let response = match route.as_str() {
            "GET /" => self.handle_get_root(),
            "POST /users" => self.handle_post_users(&request),
            _ => self.handle_not_found(),
        };

        stream.write_all(response.as_bytes()).unwrap();
    }

    fn handle_get_root(&self) -> String {
        let content = self.static_files.get("index.html").unwrap();

        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            content.len(),
            content
        )
    }

    fn handle_post_users(&mut self, request: &HttpRequest) -> String {
        let username = request.query_params.get("username").unwrap().trim();

        self.chat_room.add_user(username);

        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            username.len(),
            username
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
