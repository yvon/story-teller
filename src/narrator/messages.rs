use crate::chat::{Message, Role};
use serde::{self, Deserialize};

#[derive(Deserialize)]
pub struct ChatResponse {
    pub text: String,
    pub choices: Vec<String>,
}

pub fn parse_response(message: &Message) -> ChatResponse {
    serde_json::from_str(&message.content).unwrap()
}

pub fn user_message(content: String) -> Message {
    Message {
        role: Role::User,
        content,
    }
}
