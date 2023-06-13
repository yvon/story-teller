use crate::chat::{Message, Role, Service};
use crate::narrator::messages::parse_response;
use std::sync::Arc;

type Parent = Option<Arc<Chapter>>;

pub struct Chapter {
    text: String,
    message: Message,
    total_tokens: u32,
    choices: Vec<String>,
}

impl Chapter {
    pub async fn load(service: &Service, parent: Option<Arc<Message>>, content: String) -> Self {
        let message = Message {
            role: Role::User,
            content,
            parent,
        };

        let (response, total_tokens) = submit(&service, message.clone()).await;
        let parsed_response = parse_response(&response);

        Self {
            message,
            total_tokens,
            text: parsed_response.text,
            choices: parsed_response.choices,
        }
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn choices(&self) -> &Vec<String> {
        &self.choices
    }

    pub fn message(&self) -> Arc<Message> {
        Arc::new(self.message.clone())
    }

    pub fn total_tokens(&self) -> u32 {
        self.total_tokens
    }
}

async fn submit(service: &Service, message: Message) -> (Message, u32) {
    let messages = message.messages();
    let api_response = service.submit(&messages).await;
    let response_message = api_response.choices.get(0).unwrap().message.clone();
    let total_tokens = api_response.usage.total_tokens;

    eprintln!("Total tokens: {}", total_tokens);
    (response_message, total_tokens)
}
