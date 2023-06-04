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
    service: Service,
    messages: Vec<Message>,
}

impl BasicNarrator {
    pub fn new(service: Service) -> Self {
        let initial_prompt = read_prompt("initial_prompt.txt");

        let inital_message = Message {
            role: Role::User,
            content: initial_prompt,
        };

        Self {
            service,
            messages: vec![inital_message],
        }
    }

    pub fn choose(&mut self, choice: String) {
        let new_message = Message {
            role: Role::User,
            content: choice,
        };
        self.messages.push(new_message);
    }

    // TODO: naming
    pub async fn submit(&mut self) -> Story {
        let response_message = self.service.submit(&self.messages).await.unwrap();
        self.messages.push(response_message.clone());
        serde_json::from_str(&response_message.content).unwrap()
    }

    // TODO: it's a POC, remove or rework
    pub async fn summarize(&self) -> String {
        let mut messages = self.messages.clone();

        messages.push(Message {
            role: Role::User,
            content: read_prompt("summarize.txt"),
        });

        let response_message = self.service.submit(&messages).await.unwrap();
        let json_response: SummaryResponse =
            serde_json::from_str(&response_message.content).unwrap();
        json_response.summary
    }
}

fn read_prompt(path: &'static str) -> String {
    std::fs::read_to_string(path).expect(&format!("Failed to read initial prompt from {}", path))
}
