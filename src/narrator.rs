use crate::chat::{Message, Role, Service};
use serde::Deserialize;
use tokio::task::{self, JoinHandle};

const TOKEN_THRESHOLD_FOR_SUMMARY: u32 = 1000;
const TOKEN_THRESHOLD_FOR_REDUCE: u32 = 1500;

#[derive(Deserialize)]
pub struct Story {
    pub text: String,
    pub choices: Vec<String>,
}

#[derive(Deserialize)]
struct SummaryResponse {
    pub summary: String,
}

pub struct BasicNarrator {
    story: Option<Story>,
    messages: Vec<Message>,
    service: Service,
    summarize_handle: Option<JoinHandle<(usize, String)>>,
    total_tokens: u32,
}

impl BasicNarrator {
    pub fn new(service: Service) -> Self {
        Self {
            messages: vec![initial_message()],
            story: None,
            service,
            summarize_handle: None,
            total_tokens: 0,
        }
    }

    pub async fn start(&mut self) {
        let (response_message, total_tokens) = self.submit(&self.messages).await;

        self.total_tokens = total_tokens;
        self.story = Some(parse_story(&response_message));
        self.messages.push(response_message);
    }

    pub fn story(&self) -> &Option<Story> {
        &self.story
    }

    pub async fn choose(&mut self, choice: String) {
        let message = Message {
            role: Role::User,
            content: choice,
        };

        self.messages.push(message);
        let (response_message, total_tokens) = self.submit(&self.messages).await;

        self.total_tokens = total_tokens;
        self.story = Some(parse_story(&response_message));
        self.messages.push(response_message);
    }

    pub async fn post_processing(&mut self) {
        if self.summarize_handle.is_none() {
            if self.total_tokens > TOKEN_THRESHOLD_FOR_SUMMARY {
                self.start_summarization()
            }
        } else if self.total_tokens > TOKEN_THRESHOLD_FOR_REDUCE {
            self.reduce_messages().await;
        }
    }

    async fn submit(&self, messages: &Vec<Message>) -> (Message, u32) {
        let api_response = self.service.submit(messages).await;
        let response_message = api_response.choices.get(0).unwrap().message.clone();
        let total_tokens = api_response.usage.total_tokens;

        eprintln!("Total tokens: {}", self.total_tokens);

        (response_message, total_tokens)
    }

    fn start_summarization(&mut self) {
        let future = summarize(self.messages.clone());
        self.summarize_handle = Some(task::spawn(future));
    }

    async fn reduce_messages(&mut self) {
        let handle = self.summarize_handle.take().unwrap();
        let (size, summary) = handle.await.unwrap();
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

fn parse_story(message: &Message) -> Story {
    serde_json::from_str(&message.content).unwrap()
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
