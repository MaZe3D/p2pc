use chrono::{DateTime, Local};
use uuid::Uuid;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Message {
    sender: String,
    message_id: Uuid,
    content: String,
    answer_to: Option<uuid::Uuid>,
    pub recieved_time: DateTime<Local>,
}

impl Message {
    pub fn new(sender_id: String, message: String, answer_to: Option<Uuid>) -> Self {
        Self {
            sender: sender_id,
            message_id: uuid::Uuid::new_v4(),
            content: message,
            answer_to: answer_to,
            recieved_time: Local::now(),
        }
    }

    pub fn get_sender(&self) -> &String {
        &self.sender
    }

    pub fn get_content(&self) -> &String {
        &self.content
    }

    pub fn get_message_id(&self) -> &uuid::Uuid {
        &self.message_id
    }

    pub fn get_answer_to(&self) -> Option<&uuid::Uuid> {
        self.answer_to.as_ref()
    }
}