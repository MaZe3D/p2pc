#[derive(serde::Deserialize, serde::Serialize)]
pub struct Message {
    sender: String,
    message_id: uuid::Uuid,
    content: String,
    answer_to: Option<uuid::Uuid>,
}

impl Message {
    pub fn new(sender_id: String, message: String) -> Self {
        Self {
            sender: sender_id,
            message_id: uuid::Uuid::new_v4(),
            content: message,
            answer_to: None,
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

    pub fn answer_to(&mut self, message_id: uuid::Uuid) {
        self.answer_to = Some(message_id);
    }
}