use uuid::Uuid;
mod message;
pub use message::*;

mod contact;
pub use contact::*;

mod contacts;
pub use contacts::*;

mod chats;
pub use chats::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Chat {
    chat_id: Uuid,
    pub name: String,
    messages: Vec<Message>,
    participants: Vec<String>,
}

impl Chat {
    pub fn send_message(
        &mut self,
        p2pc: &mut p2pc_lib::P2pc,
        sender_id: String,
        message: String,
        answer_to: Option<Uuid>,
    ) {
        let ui_message = Message::new(sender_id, message.clone(), answer_to);
        if p2pc
            .execute(p2pc_lib::Action::SendMessage(p2pc_lib::ChatMessage {
                participants: self.participants.clone(),
                content: message,
                id: *ui_message.get_message_id(),
                chat_id: self.chat_id,
                answer_to,
            }))
            .is_ok()
        {
            self.messages.push(ui_message);
        }
    }

    pub fn insert_message(&mut self, sender_id: String, message: String, answer_to: Option<Uuid>, message_id: Uuid) {
        self.messages
            .push(Message::new_with_id(sender_id, message.clone(), answer_to, message_id));
    }

    pub fn new_chat(participants: Vec<String>) -> Self {
        Self {
            chat_id: Uuid::new_v4(),
            messages: Vec::new(),
            name: "New Chat".to_string(),
            participants,
        }
    }

    pub fn new_incoming_chat(participants: Vec<String>, chat_id: Uuid) -> Self {
        Self {
            chat_id,
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
        self.messages
            .iter()
            .find(|message| message.get_message_id() == message_id)
    }

    pub fn get_participants(&self) -> &Vec<String> {
        &self.participants
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
