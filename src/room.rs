use std::{collections::HashSet, fmt::Display};

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

pub struct ChatRoom {
    active_users: HashSet<String>,
    messages: Vec<Message>,
}

impl ChatRoom {
    pub fn new() -> Self {
        ChatRoom {
            active_users: HashSet::new(),
            messages: vec![],
        }
    }

    pub fn add_user(&mut self, username: &str) {
        self.active_users.insert(username.to_string());
    }

    pub fn remove_user(&mut self, username: &str) {
        self.active_users.remove(&username.to_string());
    }

    pub fn get_active_users(&self) -> Vec<&String> {
        let mut users = self.active_users.iter().collect::<Vec<_>>();
        users.sort();
        users
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message)
    }

    pub fn get_messages(&self) -> Vec<Message> {
        let mut messages = self.messages.to_vec();
        messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        messages
    }
}

#[cfg(test)]
mod tests {
    use super::{ChatRoom, Message};

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

        let actual = room.get_messages();
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
