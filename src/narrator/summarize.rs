use super::{LinkedMessage, SharedMessage};
use crate::chat::{request, Message, Role, Service};
use serde::{self, Deserialize};

const TOKEN_THRESHOLD_FOR_SUMMARY: u32 = 1000;

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

pub fn message_above_threshold(mut message: SharedMessage) -> Option<SharedMessage> {
    let mut selected: Option<SharedMessage> = None;

    loop {
        let (total_tokens, parent) = {
            let read_lock = message.read();
            (read_lock.total_tokens, read_lock.parent.clone())
        };

        match total_tokens {
            Some(value) if value < TOKEN_THRESHOLD_FOR_SUMMARY => break,
            Some(_) => selected = Some(message),
            None => (),
        }

        match parent {
            Some(value) => message = value,
            None => break,
        }
    }

    selected
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
        total_tokens: None,
    };

    let body = request::Body {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_message() -> Message {
        Message {
            role: Role::User,
            content: Some("Hello, world!".to_string()),
            name: None,
            function_call: None,
        }
    }

    fn linked_messages(total_tokens_collection: Vec<Option<u32>>) -> SharedMessage {
        let mut parent: Option<SharedMessage> = None;

        for total_tokens in total_tokens_collection.iter() {
            parent = Some(SharedMessage::new(dummy_message(), parent, *total_tokens));
        }

        parent.unwrap()
    }

    #[test]
    fn it_finds_the_message_above_threshold() {
        let msg = linked_messages(vec![Some(100), Some(1200), None, Some(2000)]);
        let selected = message_above_threshold(msg);

        let total_tokens = selected
            .expect("selected message")
            .read()
            .total_tokens
            .expect("total_tokens");

        assert_eq!(total_tokens, 1200);
    }

    #[test]
    fn it_returns_none_witout_message_above_threshold() {
        let msg = linked_messages(vec![Some(100), Some(200)]);
        let selected = message_above_threshold(msg);

        assert!(selected.is_none());
    }
}
