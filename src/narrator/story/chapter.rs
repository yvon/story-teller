use crate::chat::{request, Message, Role, Service};
use crate::narrator::linked_messages::{LinkedMessage, SharedMessage};
use serde::{self, Deserialize};
use std::sync::{Arc, RwLock};

static MAX_ATTEMPTS: u32 = 3;

pub struct Chapter {
    text: String,
    message: SharedMessage,
    total_tokens: u32,
    choices: Vec<String>,
}

struct Request {
    message: LinkedMessage,
}

#[derive(Deserialize)]
struct ChatResponse {
    pub text: String,
    pub choices: Vec<String>,
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

impl Request {
    fn new(parent: Option<SharedMessage>, content: String) -> Self {
        Self {
            message: LinkedMessage {
                message: Message {
                    role: Role::User,
                    content: Some(content),
                    name: None,
                    function_call: None,
                },
                parent,
            },
        }
    }

    async fn perform(&self, service: &Service) -> (ChatResponse, u32) {
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
