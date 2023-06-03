use crate::chat::{Message, Role, Service};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Story {
    pub text: String,
    pub choices: Vec<String>,
}

#[derive(Deserialize)]
pub struct SummaryResponse {
    pub summary: String,
}

pub struct BasicNarrator {
    initial_prompt: String,
}

impl BasicNarrator {
    pub fn new() -> Self {
        Self {
            initial_prompt: read_prompt("initial_prompt.txt"),
        }
    }

    pub fn initial_prompt(&self) -> &String {
        &self.initial_prompt
    }
}

pub fn parse_chat_message(message: &String) -> Result<Story, serde_json::Error> {
    serde_json::from_str(message)
}

pub async fn summarize(service: &Service, messages: &Vec<Message>) -> String {
    let mut messages = messages.clone();
    messages.push(Message {
        role: Role::User,
        content: read_prompt("summarize.txt"),
    });
    let response_message = service.submit(&messages).await.unwrap();
    let SummaryResponse { summary } = serde_json::from_str(&response_message.content).unwrap();
    summary
}

fn read_prompt(path: &'static str) -> String {
    std::fs::read_to_string(path).expect(&format!("Failed to read initial prompt from {}", path))
}
