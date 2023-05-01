use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    ops::Deref,
    sync::{Arc, Mutex},
};

pub trait State {
    fn add_user(&self, username: &str);
    fn remove_user(&self, username: &str);
    fn online_users(&self) -> Vec<String>;
    fn offline_users(&self) -> Vec<String>;
    fn add_message(&self, message: Message);
    fn messages(&self) -> Vec<Message>;
}

pub struct ThreadSafeChatRoom(Arc<Mutex<ChatRoom>>);

impl ThreadSafeChatRoom {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(ChatRoom::new())))
    }
}

impl Clone for ThreadSafeChatRoom {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl State for ThreadSafeChatRoom {
    fn add_user(&self, username: &str) {
        self.0.lock().unwrap().add_user(username);
    }

    fn remove_user(&self, username: &str) {
        self.0.lock().unwrap().remove_user(username);
    }

    fn online_users(&self) -> Vec<String> {
        self.0.lock().unwrap().online_users()
    }

    fn offline_users(&self) -> Vec<String> {
        self.0.lock().unwrap().offline_users()
    }

    fn add_message(&self, message: Message) {
        self.0.lock().unwrap().add_message(message);
    }

    fn messages(&self) -> Vec<Message> {
        self.0.lock().unwrap().messages()
    }
}

struct ChatRoom {
    online_users: HashSet<String>,
    offline_users: HashSet<String>,
    messages: Vec<Message>,
}

impl ChatRoom {
    fn new() -> Self {
        ChatRoom {
            online_users: HashSet::new(),
            offline_users: HashSet::new(),
            messages: Vec::new(),
        }
    }

    fn add_user(&mut self, username: &str) {
        let username = username.trim().to_string();
        self.offline_users.remove(&username);
        self.online_users.insert(username);
    }

    fn remove_user(&mut self, username: &str) {
        let username = username.trim().to_string();
        self.online_users.remove(&username);
        self.offline_users.insert(username);
    }

    fn online_users(&self) -> Vec<String> {
        sort_users(&self.online_users)
    }

    fn offline_users(&self) -> Vec<String> {
        sort_users(&self.offline_users)
    }

    fn add_message(&mut self, message: Message) {
        self.messages.push(message)
    }

    fn messages(&self) -> Vec<Message> {
        let mut messages = self.messages.to_vec();
        messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        messages
    }
}

fn sort_users(users: &HashSet<String>) -> Vec<String> {
    let mut users = users.iter().map(ToString::to_string).collect::<Vec<_>>();
    users.sort();
    users
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub timestamp: u128,
    pub sender: String,
    pub content: String,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\"timestamp\":{},\"sender\":\"{}\",\"content\":\"{}\"}}",
            self.timestamp, self.sender, self.content
        )
    }
}

pub struct StaticFiles(Arc<HashMap<String, String>>);

impl StaticFiles {
    pub fn new() -> Self {
        let mut static_files = HashMap::new();

        let paths = std::fs::read_dir("static/").unwrap();
        for path in paths {
            let path = path.unwrap().path();
            let index = std::fs::read_to_string(&path).unwrap();
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            static_files.insert(filename, index);
        }

        Self(Arc::new(static_files))
    }
}

impl Clone for StaticFiles {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl Deref for StaticFiles {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Message, State, ThreadSafeChatRoom};

    #[test]
    fn it_sorts_messages_by_timestamp() {
        let room = ThreadSafeChatRoom::new();
        room.add_message(Message {
            timestamp: 2,
            sender: String::from("user"),
            content: String::from("test"),
        });
        room.add_message(Message {
            timestamp: 1,
            sender: String::from("user"),
            content: String::from("test"),
        });

        let actual = room.messages();
        let expected = vec![
            Message {
                timestamp: 1,
                sender: String::from("user"),
                content: String::from("test"),
            },
            Message {
                timestamp: 2,
                sender: String::from("user"),
                content: String::from("test"),
            },
        ];

        assert_eq!(actual, expected);
    }
}
