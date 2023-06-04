use crate::chat::{ApiResponse, Message, Role, Service};
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
    story: Story,
}

impl BasicNarrator {
    pub async fn new(service: Service) -> Self {
        let mut messages = vec![initial_message()];
        let story = submit(&service, &mut messages).await;

        Self {
            service,
            messages,
            story,
        }
    }

    pub fn story(&self) -> &Story {
        &self.story
    }

    pub async fn choose(&mut self, choice: String) {
        let new_message = Message {
            role: Role::User,
            content: choice,
        };
        self.messages.push(new_message);
        self.story = submit(&self.service, &mut self.messages).await;
    }

    // TODO: it's a POC, remove or rework
    pub async fn summarize(&self) -> String {
        let mut messages = self.messages.clone();

        messages.push(Message {
            role: Role::User,
            content: read_prompt("summarize.txt"),
        });

        let response_message = self.service.submit_and_return_message(&messages).await;
        let json_response: SummaryResponse =
            serde_json::from_str(&response_message.content).unwrap();
        json_response.summary
    }
}

fn initial_message() -> Message {
    let initial_prompt = read_prompt("initial_prompt.txt");

    Message {
        role: Role::User,
        content: initial_prompt,
    }
}

fn read_prompt(path: &'static str) -> String {
    std::fs::read_to_string(path).expect(&format!("Failed to read initial prompt from {}", path))
}

async fn submit(service: &Service, messages: &mut Vec<Message>) -> Story {
    let api_response = service.submit(messages).await;
    let response_message = api_response.choices.get(0).unwrap().message.clone();

    eprintln!("Spent tokens: {}", api_response.usage.total_tokens);
    messages.push(response_message.clone());
    serde_json::from_str(&response_message.content).unwrap()
}
