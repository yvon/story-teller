use crate::chat::{Message, Role, Service};
use crate::narrator::read_prompt;
use serde::Deserialize;
use tokio::task::{self, JoinHandle};

const TOKEN_THRESHOLD_FOR_SUMMARY: u32 = 1000;
const TOKEN_THRESHOLD_FOR_REDUCE: u32 = 1500;

pub struct MessageManager {
    messages: Vec<Message>,
    service: Service,
    summarize_handle: Option<JoinHandle<(usize, String)>>,
    total_tokens: u32,
}

impl MessageManager {
    pub fn new(initial_message: Message) -> Self {
        Self {
            messages: vec![initial_message],
            service: Service::new(),
            summarize_handle: None,
            total_tokens: 0,
        }
    }

    pub async fn submit(&mut self) -> Message {
        let api_response = self.service.submit(&self.messages).await;
        let response_message = api_response.choices.get(0).unwrap().message.clone();

        self.total_tokens = api_response.usage.total_tokens;
        eprintln!("Total tokens: {}", self.total_tokens);

        self.messages.push(response_message.clone());
        response_message
    }

    pub async fn reply(&mut self, message: Message) -> Message {
        self.messages.push(message);
        self.submit().await
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

    fn start_summarization(&mut self) {
        let future = summarize(self.messages.clone());
        self.summarize_handle = Some(task::spawn(future));
    }

    async fn reduce_messages(&mut self) {
        let handle = self.summarize_handle.take().unwrap();
        let (size, summary) = handle.await.unwrap();
        let prompt = format!(include_str!("reduce.txt"), summary);
        let new_tail = vec![Message {
            role: Role::User,
            content: prompt,
        }];
        self.messages.splice(..size, new_tail);
    }
}

#[derive(Deserialize)]
struct SummaryResponse {
    pub summary: String,
}

pub async fn summarize(mut messages: Vec<Message>) -> (usize, String) {
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
