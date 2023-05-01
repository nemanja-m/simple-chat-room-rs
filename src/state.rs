use std::{collections::HashSet, fmt::Display};

pub trait State {
    fn add_user(&mut self, username: &str);
    fn remove_user(&mut self, username: &str);
    fn online_users(&self) -> Vec<&String>;
    fn offline_users(&self) -> Vec<&String> {
        vec![]
    }
    fn add_message(&mut self, message: Message);
    fn messages(&self) -> Vec<Message>;
}

pub struct ChatRoom {
    online_users: HashSet<String>,
    messages: Vec<Message>,
}

impl ChatRoom {
    pub fn new() -> Self {
        ChatRoom {
            online_users: HashSet::new(),
            messages: vec![],
        }
    }
}

impl State for ChatRoom {
    fn add_user(&mut self, username: &str) {
        self.online_users.insert(username.to_string());
    }

    fn remove_user(&mut self, username: &str) {
        self.online_users.remove(&username.to_string());
    }

    fn online_users(&self) -> Vec<&String> {
        let mut users = self.online_users.iter().collect::<Vec<_>>();
        users.sort();
        users
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

#[cfg(test)]
mod tests {
    use super::{ChatRoom, Message, State};

    #[test]
    fn it_sorts_messages_by_timestamp() {
        let mut room = ChatRoom::new();
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
