use uuid::Uuid;
mod message;
pub use message::*;

mod contact;
pub use contact::*;

mod contacts;
pub use contacts::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Chat {
    chat_id: Uuid,
    pub name: String,
    messages: Vec<Message>,
    participants: Vec<String>,
}

impl Chat {
    pub fn new_message(&mut self, sender_id: String, message: String, answer_to: Option<Uuid>) {
        self.messages.push(Message::new(sender_id, message, answer_to));
    }

    pub fn new_chat(participants: Vec<String>) -> Self {
        Self {
            chat_id: Uuid::new_v4(),
            messages: Vec::new(),
            name: "New Chat".to_string(),
            participants,
        }
    }

    pub fn get_chat_messages(&self) -> &Vec<Message> {
        &self.messages
    }

    pub fn get_chat_id(&self) -> &Uuid {
        &self.chat_id
    }

    pub fn get_message_from_id(&self, message_id: &Uuid) -> Option<&Message> {
        self.messages.iter().find(|message| message.get_message_id() == message_id)
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
    pub fn from_chat(chat: &Chat) -> Self {
        Self {
            name: chat.name.clone(),
            participants: chat.participants.clone(),
        }
    }
}