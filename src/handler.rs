use std::time::{SystemTime, UNIX_EPOCH};

use log::info;

use crate::request::HttpRequest;
use crate::state::{Message, State};

pub fn handle_request<T: State>(request: &HttpRequest<T>) -> String {
    let route = request.route();

    let response = match route.as_str() {
        "GET /" => handle_get_root(request),
        "GET /users" => handle_get_users(&request),
        "GET /messages" => handle_get_messages(&request),
        "POST /messages" => handle_post_messages(&request),
        "POST /chat" => handle_enter_chat(&request),
        "POST /chat/exit" => handle_exit_chat(&request),
        _ => handle_static_files(&request),
    };

    response
}

fn handle_get_root<T: State>(request: &HttpRequest<T>) -> String {
    let content = request.static_files.get("login.html").unwrap();

    format_http_response(200, "OK", content, "text/html")
}

fn handle_get_users<T: State>(request: &HttpRequest<T>) -> String {
    let online_users = users_to_json_array(&request.state.online_users());
    let offline_users = users_to_json_array(&request.state.offline_users());

    let content = format!(
        "{{\"users\": {{\"online\": [{}], \"offline\": [{}]}}}}",
        online_users.trim(),
        offline_users.trim()
    );

    format_http_response(200, "OK", &content, "application/json")
}

fn users_to_json_array(users: &Vec<String>) -> String {
    let users = users
        .iter()
        .map(|user| format!("\"{}\"", user))
        .collect::<Vec<_>>()
        .join(",");

    format!("[{}]", users)
}

fn handle_get_messages<T: State>(request: &HttpRequest<T>) -> String {
    let messages = request
        .state
        .messages()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");

    let content = format!("{{\"messages\": [{}]}}", messages.trim());

    format_http_response(200, "OK", &content, "application/json")
}

fn handle_post_messages<T: State>(request: &HttpRequest<T>) -> String {
    let sender = request.form_data.get("sender").unwrap().trim().to_string();

    let content = request.form_data.get("content").unwrap().trim().to_string();

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

    request.state.add_message(message);

    format_http_response(200, "OK", "", "text/plain")
}

fn handle_enter_chat<T: State>(request: &HttpRequest<T>) -> String {
    let username = request.form_data.get("username").unwrap().trim();
    request.state.add_user(username);

    info!("User {username} entered chat room");

    let content = request.static_files.get("chat.html").unwrap();

    format_http_response(200, "OK", content, "text/html")
}

fn handle_exit_chat<T: State>(request: &HttpRequest<T>) -> String {
    let username = request.form_data.get("username").unwrap().trim();
    request.state.remove_user(username);

    info!("User {username} exited chat room");

    format_http_response(200, "OK", "", "text/plain")
}

fn handle_static_files<T: State>(request: &HttpRequest<T>) -> String {
    let key = request.path.replace("/", "");

    if request.static_files.contains_key(&key) {
        let content = request.static_files.get(&key).unwrap();

        format_http_response(200, "OK", content, "")
    } else {
        let content = request.static_files.get("404.html").unwrap();

        format_http_response(404, "Not Found", content, "text/html")
    }
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
