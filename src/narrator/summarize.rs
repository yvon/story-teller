use crate::chat::{Message, Role, Service};
use crate::narrator::Chapter;
use serde::{self, Deserialize};
use std::sync::Arc;

const TOKEN_THRESHOLD_FOR_SUMMARY: u32 = 1000;
const TOKEN_THRESHOLD_FOR_REDUCE: u32 = 1500;

pub struct SummarizedChapter {
    chapter: Arc<Chapter>,
    content: String,
}

#[derive(Deserialize)]
struct SummaryResponse {
    pub summary: String,
}

impl SummarizedChapter {
    pub async fn new(service: Service, chapter: Arc<Chapter>) -> Self {
        let messages = chapter.as_ref().messages();

        Self {
            chapter,
            content: summarize(&service, messages).await,
        }
    }
}

async fn summarize(service: &Service, mut messages: Vec<Message>) -> String {
    messages.push(Message {
        role: Role::User,
        content: include_str!("summarize.txt").to_string(),
    });

    let response_message = service.submit_and_return_message(&messages).await;
    let json_response: SummaryResponse = serde_json::from_str(&response_message.content).unwrap();

    eprintln!("SUMMARY: {}", json_response.summary);
    json_response.summary
}
