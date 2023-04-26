use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct User(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub ts: usize,
    pub user: User,
    pub message: String,
}

pub struct ChatRoom {
    active_users: HashSet<User>,
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
        self.active_users.insert(User(username.to_string()));
    }

    pub fn remove_user(&mut self, username: &str) {
        self.active_users.remove(&User(username.to_string()));
    }

    pub fn get_active_users(&self) -> &HashSet<User> {
        &self.active_users
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message)
    }

    pub fn get_messages(&self) -> Vec<Message> {
        let mut messages = self.messages.to_vec();
        messages.sort_by(|a, b| b.ts.cmp(&a.ts));
        messages
    }
}

#[cfg(test)]
mod tests {
    use super::{ChatRoom, Message, User};

    #[test]
    fn it_sorts_messages_by_timestamp() {
        let mut room = ChatRoom::new();
        room.add_message(Message {
            ts: 1,
            user: User(String::from("user")),
            message: String::from("test"),
        });
        room.add_message(Message {
            ts: 2,
            user: User(String::from("user")),
            message: String::from("test"),
        });

        let actual = room.get_messages();
        let expected = vec![
            Message {
                ts: 2,
                user: User(String::from("user")),
                message: String::from("test"),
            },
            Message {
                ts: 1,
                user: User(String::from("user")),
                message: String::from("test"),
            },
        ];

        assert_eq!(actual, expected);
    }
}
