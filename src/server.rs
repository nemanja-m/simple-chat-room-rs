use log::info;
use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::http::HttpRequest;
use crate::room::{ChatRoom, Message};

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

        // GET /users are polling requests.
        if &route != "GET /users" && &route != "GET /messages" {
            info!("{route}");
        }

        let response = match route.as_str() {
            "GET /" => self.handle_get_root(),
            "GET /users" => self.handle_get_users(),
            "GET /messages" => self.handle_get_messages(),
            "POST /messages" => self.handle_post_messages(&request),
            "POST /chat" => self.handle_enter_chat(&request),
            "POST /chat/exit" => self.handle_exit_chat(&request),
            _ if self
                .static_files
                .contains_key(&request.path.replace("/", "")) =>
            {
                let file = &request.path.replace("/", "");
                self.handle_static_file(file)
            }
            _ => self.handle_not_found(),
        };

        stream.write_all(response.as_bytes()).unwrap();
    }

    fn handle_get_root(&self) -> String {
        let content = self.static_files.get("login.html").unwrap();

        format_http_response(200, "OK", content, "text/html")
    }

    fn handle_get_users(&self) -> String {
        let users = self
            .chat_room
            .get_active_users()
            .iter()
            .map(|&user| format!("\"{}\"", user.clone()))
            .collect::<Vec<_>>()
            .join(",");

        let content = format!("{{\"users\": [{}]}}", users.trim());

        format_http_response(200, "OK", &content, "application/json")
    }

    fn handle_get_messages(&self) -> String {
        let messages = self
            .chat_room
            .get_messages()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");

        let content = format!("{{\"messages\": [{}]}}", messages.trim());

        format_http_response(200, "OK", &content, "application/json")
    }

    fn handle_post_messages(&mut self, request: &HttpRequest) -> String {
        let sender = request
            .query_params
            .get("sender")
            .unwrap()
            .trim()
            .to_string();

        let content = request
            .query_params
            .get("content")
            .unwrap()
            .trim()
            .to_string();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let message = Message {
            timestamp,
            sender,
            content,
        };

        info!("Message: {}", &message);

        self.chat_room.add_message(message);

        format_http_response(200, "OK", "", "text/plain")
    }

    fn handle_enter_chat(&mut self, request: &HttpRequest) -> String {
        let username = request.query_params.get("username").unwrap().trim();
        self.chat_room.add_user(username);
        info!("User {username} entered chat room");

        let content = self.static_files.get("chat.html").unwrap();

        format_http_response(200, "OK", content, "text/html")
    }

    fn handle_exit_chat(&mut self, request: &HttpRequest) -> String {
        let username = request.query_params.get("username").unwrap().trim();
        self.chat_room.remove_user(username);
        info!("User {username} exited chat room");

        format_http_response(200, "OK", "", "text/plain")
    }

    fn handle_static_file(&self, file: &str) -> String {
        let content = self.static_files.get(file).unwrap();
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

fn format_http_response(
    status_code: usize,
    message: &str,
    content: &str,
    content_type: &str,
) -> String {
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status_code,
        message,
        content_type,
        content.len(),
        content
    )
}
