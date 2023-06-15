use crate::chat::{request, Message, Role, Service};
use crate::narrator::linked_messages::{LinkedMessage, SharedMessage};
use serde::{self, Deserialize};
use std::sync::{Arc, RwLock};

pub struct Chapter {
    text: String,
    message: SharedMessage,
    total_tokens: u32,
    choices: Vec<String>,
}

#[derive(Deserialize)]
struct ChatResponse {
    pub text: String,
    pub choices: Vec<String>,
}

impl Chapter {
    pub async fn load(service: &Service, parent: Option<SharedMessage>, content: String) -> Self {
        let query = LinkedMessage {
            message: Message {
                role: Role::User,
                content: Some(content),
                name: None,
                function_call: None,
            },
            parent: parent.clone(),
        };

        let (response, total_tokens) = submit(&service, &query).await;
        let parsed_response = parse_response(&response);
        let text = parsed_response.text.clone();
        let choices = parsed_response.choices;

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

fn parameters() -> serde_json::Value {
    serde_json::from_str(include_str!("parameters_schema.json")).unwrap()
}

fn functions() -> Vec<request::Function> {
    vec![request::Function {
        name: "chapter",
        parameters: Some(parameters()),
        description: None,
    }]
}

fn body(messages: Vec<Message>) -> request::Body {
    request::Body {
        messages,
        functions: Some(functions()),
        function_call: Some(request::FunctionCall::Name("chapter")),
        ..Default::default()
    }
}

async fn submit(service: &Service, linked_message: &LinkedMessage) -> (Message, u32) {
    let body = body(linked_message.messages());
    let api_response = service.submit(body).await;
    let response_message = api_response.choices.get(0).unwrap().message.clone();
    let total_tokens = api_response.usage.total_tokens;

    eprintln!("Total tokens: {}", total_tokens);
    (response_message, total_tokens)
}

fn parse_response(message: &Message) -> ChatResponse {
    let function_call = message.function_call.as_ref().expect("No function call");

    if function_call.name != "chapter" {
        panic!("Expected a chapter function call");
    }

    serde_json::from_str(&function_call.arguments).unwrap()
}
