use crate::chat::{Body, Message, Role, Service};
use crate::narrator::linked_messages::{LinkedMessage, SharedMessage};
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

async fn summarize(service: &Service, parent: SharedMessage) -> String {
    let query = Message {
        role: Role::User,
        content: Some(include_str!("summarize.txt").to_string()),
        name: None,
        function_call: None,
    };

    let linked_message = LinkedMessage {
        message: query,
        parent: Some(parent),
    };

    let body = Body {
        messages: linked_message.messages(),
        ..Default::default()
    };

    let api_response = service.submit(body).await;
    let response_message = api_response.message();
    let json_response: SummaryResponse =
        serde_json::from_str(&response_message.content.unwrap()).unwrap();

    eprintln!("SUMMARY: {}", json_response.summary);
    json_response.summary
}
