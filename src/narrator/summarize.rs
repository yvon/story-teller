use crate::chat::{Message, Role, Service};
use serde::{self, Deserialize};
use std::sync::Arc;

pub struct Summary {
    pub message: Arc<Message>,
    content: String,
}

#[derive(Deserialize)]
struct SummaryResponse {
    pub summary: String,
}

// impl Summary {
//     pub async fn new(service: Service, message: Arc<Message>) -> Self {
//         let content = summarize(&service, message.clone()).await;
//         Self { message, content }
//     }
// }
//
// async fn summarize(service: &Service, message: Arc<Message>) -> String {
//     let query = Message {
//         role: Role::User,
//         content: include_str!("summarize.txt").to_string(),
//         parent: Some(message),
//     };
//
//     let api_response = service.submit(&query.messages()).await;
//     let response_message = api_response.message();
//     let json_response: SummaryResponse = serde_json::from_str(&response_message.content).unwrap();
//
//     eprintln!("SUMMARY: {}", json_response.summary);
//     json_response.summary
// }
