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
        let mut messages = match &parent {
            Some(chapter_ref) => collect_messages(chapter_ref.as_ref()),
            None => Vec::new(),
        };

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

    pub fn messages(&self) -> Vec<Message> {
        collect_messages(self)
    }
}

async fn submit(service: &Service, messages: &Vec<Message>) -> (Message, u32) {
    let api_response = service.submit(messages).await;
    let response_message = api_response.choices.get(0).unwrap().message.clone();
    let total_tokens = api_response.usage.total_tokens;

    eprintln!("Total tokens: {}", total_tokens);
    (response_message, total_tokens)
}

fn collect_messages(mut current_chapter: &Chapter) -> Vec<Message> {
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
