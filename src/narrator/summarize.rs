use crate::chat::{Message, Role, Service};
use serde::{self, Deserialize};

const TOKEN_THRESHOLD_FOR_SUMMARY: u32 = 1000;
const TOKEN_THRESHOLD_FOR_REDUCE: u32 = 1500;

#[derive(Deserialize)]
struct SummaryResponse {
    pub summary: String,
}

async fn summarize(service: &Service, mut messages: Vec<Message>) -> (usize, String) {
    messages.push(Message {
        role: Role::User,
        content: include_str!("summarize.txt").to_string(),
    });

    let response_message = service.submit_and_return_message(&messages).await;
    let json_response: SummaryResponse = serde_json::from_str(&response_message.content).unwrap();

    eprintln!("SUMMARY: {}", json_response.summary);
    (messages.len(), json_response.summary)
}
