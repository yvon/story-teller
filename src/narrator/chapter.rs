use crate::chat::{Message, Service};
use crate::narrator::messages::{parse_response, user_message};
use std::sync::Arc;

type Parent = Option<Arc<Chapter>>;

pub struct Chapter {
    text: String,
    query: Message,
    response: Message,
    total_tokens: u32,
    choices: Vec<String>,
    parent: Parent,
}

impl Chapter {
    pub async fn load(service: &Service, parent: Parent, content: String) -> Self {
        let mut messages = collect_messages(&parent);
        let query = user_message(content);

        messages.push(query.clone());

        let (response, total_tokens) = submit(&service, &messages).await;
        let parsed_response = parse_response(&response);

        Self {
            total_tokens,
            parent,
            query,
            response,
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
}

async fn submit(service: &Service, messages: &Vec<Message>) -> (Message, u32) {
    let api_response = service.submit(messages).await;
    let response_message = api_response.choices.get(0).unwrap().message.clone();
    let total_tokens = api_response.usage.total_tokens;

    eprintln!("Total tokens: {}", total_tokens);
    (response_message, total_tokens)
}

fn collect_messages(parent: &Parent) -> Vec<Message> {
    if parent.is_none() {
        return Vec::new();
    }

    let mut current_chapter = parent.as_ref().unwrap();
    let mut messages = Vec::new();

    while let Some(parent_chapter) = current_chapter.parent.as_ref() {
        messages.push(current_chapter.response.clone());
        messages.push(current_chapter.query.clone());
        current_chapter = parent_chapter;
    }

    messages.push(current_chapter.response.clone());
    messages.push(current_chapter.query.clone());
    messages.reverse();
    messages
}
