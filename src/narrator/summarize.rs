use crate::chat::{Message, Role, Service, SharedMessage};
use serde::{self, Deserialize};

pub struct Summary {
    pub message: SharedMessage,
    pub content: String,
}

#[derive(Deserialize)]
struct SummaryResponse {
    pub summary: String,
}

impl Summary {
    pub async fn new(service: Service, message: SharedMessage) -> Self {
        let content = summarize(&service, message.clone()).await;
        Self { message, content }
    }
}

async fn summarize(service: &Service, message: SharedMessage) -> String {
    let query = Message {
        role: Role::User,
        content: include_str!("summarize.txt").to_string(),
        parent: Some(message),
    };

    let api_response = service.submit(&query).await;
    let response_message = api_response.message();
    let json_response: SummaryResponse = serde_json::from_str(&response_message.content).unwrap();

    eprintln!("SUMMARY: {}", json_response.summary);
    json_response.summary
}
