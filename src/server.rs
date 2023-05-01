use log::info;
use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::http::HttpRequest;
use crate::pool::ThreadPool;
use crate::room::{ChatRoom, Message};

pub struct HttpServer;

impl HttpServer {
    pub fn start<A>(address: A, num_threads: usize)
    where
        A: ToSocketAddrs,
    {
        let listener = TcpListener::bind(address).unwrap();

        info!(
            "Listening at {} for connections",
            listener.local_addr().unwrap()
        );

        let pool = ThreadPool::new(num_threads);
        let chat_room = Arc::new(Mutex::from(ChatRoom::new()));
        let static_files = Arc::new(load_static_files());

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let chat_room = Arc::clone(&chat_room);
            let static_files = Arc::clone(&static_files);

            pool.execute(|| {
                handle(chat_room, static_files, stream);
            });
        }
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

fn handle(
    chat_room: Arc<Mutex<ChatRoom>>,
    static_files: Arc<HashMap<String, String>>,
    mut stream: TcpStream,
) {
    let request = HttpRequest::from(&stream);
    let route = request.route();

    // GET /users are polling requests.
    if &route != "GET /users" && &route != "GET /messages" {
        info!("{route}");
    }

    let response = match route.as_str() {
        "GET /" => handle_get_root(&static_files),
        "GET /users" => handle_get_users(chat_room),
        "GET /messages" => handle_get_messages(chat_room),
        "POST /messages" => handle_post_messages(chat_room, &request),
        "POST /chat" => handle_enter_chat(chat_room, static_files, &request),
        "POST /chat/exit" => handle_exit_chat(chat_room, &request),
        _ if static_files.contains_key(&request.path.replace("/", "")) => {
            let file = &request.path.replace("/", "");
            handle_static_file(static_files, file)
        }
        _ => handle_not_found(static_files),
    };

    stream.write_all(response.as_bytes()).unwrap();
}

fn handle_get_root(static_files: &HashMap<String, String>) -> String {
    let content = static_files.get("login.html").unwrap();

    format_http_response(200, "OK", content, "text/html")
}

fn handle_get_users(chat_room: Arc<Mutex<ChatRoom>>) -> String {
    let users = chat_room
        .lock()
        .unwrap()
        .get_active_users()
        .iter()
        .map(|&user| format!("\"{}\"", user.clone()))
        .collect::<Vec<_>>()
        .join(",");

    let content = format!("{{\"users\": [{}]}}", users.trim());

    format_http_response(200, "OK", &content, "application/json")
}

fn handle_get_messages(chat_room: Arc<Mutex<ChatRoom>>) -> String {
    let messages = chat_room
        .lock()
        .unwrap()
        .get_messages()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");

    let content = format!("{{\"messages\": [{}]}}", messages.trim());

    format_http_response(200, "OK", &content, "application/json")
}

fn handle_post_messages(chat_room: Arc<Mutex<ChatRoom>>, request: &HttpRequest) -> String {
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

    chat_room.lock().unwrap().add_message(message);

    format_http_response(200, "OK", "", "text/plain")
}

fn handle_enter_chat(
    chat_room: Arc<Mutex<ChatRoom>>,
    static_files: Arc<HashMap<String, String>>,
    request: &HttpRequest,
) -> String {
    let username = request.query_params.get("username").unwrap().trim();
    chat_room.lock().unwrap().add_user(username);
    info!("User {username} entered chat room");

    let content = static_files.get("chat.html").unwrap();

    format_http_response(200, "OK", content, "text/html")
}

fn handle_exit_chat(chat_room: Arc<Mutex<ChatRoom>>, request: &HttpRequest) -> String {
    let username = request.query_params.get("username").unwrap().trim();
    chat_room.lock().unwrap().remove_user(username);
    info!("User {username} exited chat room");

    format_http_response(200, "OK", "", "text/plain")
}

fn handle_static_file(static_files: Arc<HashMap<String, String>>, file: &str) -> String {
    let content = static_files.get(file).unwrap();
    let content_length = content.len();

    format!(
        "HTTP/1.1 200 OK\r\nContent-Length:{}\r\n\r\n{}",
        content_length, content
    )
}

fn handle_not_found(static_files: Arc<HashMap<String, String>>) -> String {
    let content = static_files.get("404.html").unwrap();
    let content_length = content.len();

    format!(
        "HTTP/1.1 404 Not Found\r\nContent-Length:{}\r\n\r\n{}",
        content_length, content
    )
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
