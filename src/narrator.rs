use crate::chat::{Message, Role, Service};
use serde::{self, Deserialize};
use tokio::stream::StreamExt;
use tokio::task::{self, JoinHandle}; // make sure you have the `stream` feature enabled for tokio

const TOKEN_THRESHOLD_FOR_SUMMARY: u32 = 1000;
const TOKEN_THRESHOLD_FOR_REDUCE: u32 = 1500;

#[derive(Deserialize)]
struct ChatResponse {
    pub text: String,
    pub choices: Vec<String>,
}

#[derive(Deserialize)]
struct SummaryResponse {
    pub summary: String,
}

struct Variant {
    new_messages: Vec<Message>,
    text: String,
    choices: Vec<String>,
    total_tokens: u32,
}

pub struct BasicNarrator {
    text: String,
    messages: Vec<Message>,
    total_tokens: u32,
    choices: Vec<String>,
    variants: Option<Vec<Variant>>,
    summarize_handle: Option<JoinHandle<(usize, String)>>,
    service: Service,
}

impl Variant {
    async fn fetch(service: Service, previous_messages: &Vec<Message>, choice: String) -> Self {
        let mut messages = previous_messages.clone();

        let user_message = Message {
            role: Role::User,
            content: choice.clone(),
        };

        messages.push(user_message.clone());
        let (response_message, total_tokens) = submit(&service, &messages).await;
        let ChatResponse { choices, text } = parse_response(&response_message);

        Self {
            new_messages: vec![user_message, response_message],
            choices,
            text,
            total_tokens,
        }
    }
}

impl BasicNarrator {
    pub async fn new(service: Service) -> Self {
        let mut messages: Vec<Message> = vec![initial_message()];
        let (response_message, total_tokens) = submit(&service, &messages).await;
        let ChatResponse { choices, text } = parse_response(&response_message);

        messages.push(response_message);

        Self {
            service,
            text,
            messages,
            total_tokens,
            choices,
            summarize_handle: None,
            variants: None,
        }
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn choices(&self) -> &Vec<String> {
        &self.choices
    }

    async fn progress(&mut self, index: usize) {
        let variant = self.variants.take().unwrap().remove(index);
        let mut new_messages = variant.new_messages;

        self.messages.extend(new_messages.drain(..));
        self.text = variant.text;
        self.choices = variant.choices;
        self.total_tokens = variant.total_tokens;
        self.variants = None;
    }

    pub async fn post_processing(&mut self) {
        self.reduction().await;

        let handles = self.choices.iter().map(|choice| {
            task::spawn(Variant::fetch(
                self.service.clone(),
                &self.messages.clone(),
                choice.clone(),
            ))
        });

        self.variants = Some(results);
    }

    pub async fn reduction(&mut self) {
        match &self.summarize_handle {
            None => {
                if self.total_tokens > TOKEN_THRESHOLD_FOR_SUMMARY {
                    self.initiate_message_reduction();
                }
            }
            Some(join_handle) => {
                if self.total_tokens > TOKEN_THRESHOLD_FOR_REDUCE {
                    self.finalize_message_reduction().await;
                }
            }
        }
    }

    // Starts the summarization process and returns a handle to the future.
    fn initiate_message_reduction(&mut self) {
        let future = summarize(self.messages.clone());
        self.summarize_handle = Some(task::spawn(future));
    }

    // Waits for the summarization to complete, retrieves the result, and updates the messages.
    async fn finalize_message_reduction(&mut self) {
        let join_handle = self.summarize_handle.take().unwrap();
        let (size, summary) = join_handle.await.unwrap();
        let prompt = format!(include_str!("reduce.txt"), summary);
        let new_head = vec![Message {
            role: Role::User,
            content: prompt,
        }];
        self.messages.splice(..size - 1, new_head);
    }
}

pub fn read_prompt(path: &'static str) -> String {
    std::fs::read_to_string(path).expect(&format!("Failed to read initial prompt from {}", path))
}

fn initial_message() -> Message {
    let initial_prompt = read_prompt("initial_prompt.txt");

    Message {
        role: Role::User,
        content: initial_prompt,
    }
}

fn parse_response(message: &Message) -> ChatResponse {
    serde_json::from_str(&message.content).unwrap()
}

async fn submit(service: &Service, messages: &Vec<Message>) -> (Message, u32) {
    let api_response = service.submit(messages).await;
    let response_message = api_response.choices.get(0).unwrap().message.clone();
    let total_tokens = api_response.usage.total_tokens;

    eprintln!("Total tokens: {}", total_tokens);
    (response_message, total_tokens)
}

async fn summarize(mut messages: Vec<Message>) -> (usize, String) {
    let service = Service::new();

    messages.push(Message {
        role: Role::User,
        content: read_prompt("summarize.txt"),
    });

    let response_message = service.submit_and_return_message(&messages).await;
    let json_response: SummaryResponse = serde_json::from_str(&response_message.content).unwrap();

    eprintln!("SUMMARY: {}", json_response.summary);
    (messages.len(), json_response.summary)
}
