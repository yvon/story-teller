use super::{LinkedMessage, SharedMessage};
use crate::chat::{request, Message, Role, Service};
use serde::{self, Deserialize};

static MAX_ATTEMPTS: u32 = 3;

pub struct Request {
    message: LinkedMessage,
}

#[derive(Deserialize)]
pub struct ChatResponse {
    pub text: String,
    pub choices: Vec<String>,
}

impl Request {
    pub fn new(parent: Option<SharedMessage>, content: String) -> Self {
        Self {
            message: LinkedMessage {
                message: Message {
                    role: Role::User,
                    content: Some(content),
                    name: None,
                    function_call: None,
                },
                parent,
                total_tokens: None,
            },
        }
    }

    pub async fn perform(&self, service: &Service) -> (ChatResponse, u32) {
        let mut attempts = 0;

        while attempts < MAX_ATTEMPTS {
            match self.perform_once(service).await {
                Ok(value) => return value,
                Err(error) => eprintln!("Error: {}", error),
            }

            attempts += 1;
        }

        panic!("Max attempts reached");
    }

    async fn perform_once(&self, service: &Service) -> Result<(ChatResponse, u32), String> {
        let (response, total_tokens) = submit(service, &self.message).await;

        match parse_response(&response) {
            Ok(value) => Ok((value, total_tokens)),
            Err(error) => Err(error.to_string()),
        }
    }
}

fn parameters() -> serde_json::Value {
    serde_json::from_str(include_str!("parameters_schema.json")).unwrap()
}

fn functions() -> Vec<request::Function> {
    let function =
        request::Function::new("chapter", None, Some(parameters())).expect("Invalid JSON schema");

    vec![function]
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

fn parse_response(message: &Message) -> serde_json::Result<ChatResponse> {
    let function_call = message.function_call.as_ref().expect("No function call");

    if function_call.name != "chapter" {
        panic!("Expected a chapter function call");
    }

    serde_json::from_str(&function_call.arguments)
}
