use super::Chat;
use std::collections::HashMap;

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct Chats {
    chats: HashMap<uuid::Uuid, Chat>,
}

impl Chats {
    pub fn add_chat(&mut self, chat: Chat) {
        self.chats.insert(chat.chat_id, chat);
    }

    pub fn get_chat(&self, chat_id: &uuid::Uuid) -> Option<&Chat> {
        self.chats.get(&chat_id)
    }

    pub fn get_chats(&self) -> &HashMap<uuid::Uuid, Chat> {
        &self.chats
    }

    pub fn remove_chat(&mut self, chat_id: &uuid::Uuid) -> Option<Chat> {
        self.chats.remove(chat_id)
    }

    pub fn send_message(
        &mut self,
        chat_id: &uuid::Uuid,
        p2pc: &mut p2pc_lib::P2pc,
        sender_id: String,
        message: String,
        answer_to: Option<uuid::Uuid>,
    ) {
        if let Some(chat) = self.chats.get_mut(&chat_id) {
            chat.send_message(p2pc, sender_id, message, answer_to);
        }
    }
}
