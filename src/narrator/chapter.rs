use super::{Request, SharedMessage};
use crate::chat::{Message, Role, Service};

pub struct Chapter {
    text: String,
    message: SharedMessage,
    choices: Vec<String>,
}

impl Chapter {
    pub async fn load(service: &Service, parent: Option<SharedMessage>, content: String) -> Self {
        let request = Request::new(parent.clone(), content);
        let (parsed_response, total_tokens) = request.perform(service).await;
        let text = parsed_response.text.clone();
        let choices = parsed_response.choices;

        // I've chosen to recreate the message because there is a bug with OpenAI API: it doesn't
        // accept messages without content attribute. It should also helps reducing the number of
        // spent tokens.
        let message = SharedMessage::new(
            Message {
                role: Role::Assistant,
                content: Some(text.clone()),
                name: None,
                function_call: None,
            },
            // Linking to the parent and not the query message I discard the user choices. I
            // believe they do not provide value to the context.
            parent,
            Some(total_tokens),
        );

        Self {
            message,
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
}
