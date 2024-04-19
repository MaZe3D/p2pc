use uuid::Uuid;
mod message;
pub use message::*;

mod contact;
pub use contact::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Chat {
    chat_id: Uuid,
    pub name: String,
    messages: Vec<Message>,
    participants: Vec<String>,
}

impl Chat {
    pub fn new_message(&mut self, sender_id: String, message: String) {
        self.messages.push(Message::new(sender_id, message));
    }

    pub fn new_chat(participants: Vec<String>) -> Self {
        Self {
            chat_id: Uuid::new_v4(),
            messages: Vec::new(),
            name: "New Chat".to_string(),
            participants: participants,
        }
    }

    pub fn get_participants(&self) -> &Vec<String> {
        &self.participants
    }

    pub fn from_chat_window(chat_window: ChatEditWindowContent) -> Self {
        let mut chat = Self::new_chat(chat_window.participants);
        chat.name = chat_window.name;
        chat
    }

    pub fn get_chat_messages(&self) -> &Vec<Message> {
        &self.messages
    }
}

impl Default for ChatEditWindowContent {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            participants: Vec::new(),
        }
    }
}

pub struct ChatEditWindowContent {
    pub name: String,
    pub participants: Vec<String>,
}

impl ChatEditWindowContent {
    pub fn new() -> Self {
        Self {
            name: "New Chat".to_string(),
            participants: Vec::new(),
        }
    }

    pub fn from_chat(chat: &Chat) -> Self {
        Self {
            name: chat.name.clone(),
            participants: chat.participants.clone(),
        }
    }
}