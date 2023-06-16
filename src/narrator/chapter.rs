use super::{LinkedMessage, Request, SharedMessage};
use crate::chat::{Message, Role, Service};
use std::sync::{Arc, RwLock};

pub struct Chapter {
    text: String,
    message: SharedMessage,
    total_tokens: u32,
    choices: Vec<String>,
}

impl Chapter {
    pub async fn load(service: &Service, parent: Option<SharedMessage>, content: String) -> Self {
        let request = Request::new(parent.clone(), content);
        let (parsed_response, total_tokens) = request.perform(service).await;
        let text = parsed_response.text.clone();
        let choices = choices(parsed_response.choices);

        // I've chosen to recreate the message because there is a bug with OpenAI API: it doesn't
        // accept messages without content attribute. It should also helps reducing the number of
        // spent tokens.
        let linked_message = LinkedMessage {
            message: Message {
                role: Role::Assistant,
                content: Some(text.clone()),
                name: None,
                function_call: None,
            },
            // Linking to the parent and not the query message I discard the user choices. I
            // believe they do not provide value to the context.
            parent,
        };

        Self {
            message: Arc::new(RwLock::new(linked_message)),
            total_tokens,
            text,
            choices,
        }
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn choices(&self) -> &Vec<String> {
        &self.choices
    }

    pub fn message(&self) -> &SharedMessage {
        &self.message
    }

    pub fn total_tokens(&self) -> u32 {
        self.total_tokens
    }
}

fn choices(as_returned_by_chat_gpt: Vec<String>) -> Vec<String> {
    if as_returned_by_chat_gpt.len() < 1 {
        vec![include_str!("default_choice.txt").to_string()]
    } else {
        as_returned_by_chat_gpt
    }
}
