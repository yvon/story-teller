use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{Arc, RwLock};

pub type SharedMessage = Arc<RwLock<Message>>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(skip)]
    pub parent: Option<SharedMessage>,
}

#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub total_tokens: u32,
}

#[derive(Clone)]
pub struct Service {
    client: reqwest::Client,
    api_key: String,
}

impl Service {
    pub fn new() -> Self {
        let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    pub async fn submit(&self, message: &Message) -> ApiResponse {
        eprintln!("SENDING {:?}", message);
        let messages = &message.messages();

        let response = self
            .request(messages)
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json::<ApiResponse>()
            .await
            .unwrap();

        eprintln!("RESPONSE {:?}", response);
        response
    }

    async fn request(&self, messages: &Vec<Message>) -> reqwest::Result<reqwest::Response> {
        self.client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({
                "model": "gpt-3.5-turbo",
                "messages": messages
            }))
            .send()
            .await
    }
}

impl Message {
    pub fn messages(&self) -> Vec<Message> {
        let mut messages = Vec::new();
        let mut message = self.clone();

        loop {
            let parent = message.parent.clone();
            messages.push(message);

            match parent {
                None => break,
                Some(reference) => {
                    message = reference.as_ref().read().unwrap().clone();
                }
            }
        }

        messages.reverse();
        messages
    }
}

impl ApiResponse {
    pub fn message(&self) -> Message {
        self.choices.get(0).unwrap().message.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_deserialization() {
        let json = r#"{
            "role": "user",
            "content": "Hello, world!"
        }"#;

        let message: Message = serde_json::from_str(json).unwrap();

        assert_eq!(message.role, Role::User);
        assert_eq!(message.content, "Hello, world!");
    }

    #[test]
    fn test_message_serialization() {
        let message = Message {
            role: Role::User,
            content: "Hello, world!".to_string(),
            parent: None,
        };

        let json = serde_json::to_string(&message).unwrap();

        assert_eq!(json, r#"{"role":"user","content":"Hello, world!"}"#);
    }
}
