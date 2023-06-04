use serde::{Deserialize, Serialize};
use std::env;

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

    pub async fn submit_and_return_message(&self, messages: &Vec<Message>) -> Message {
        let api_response = self.submit(messages).await;
        api_response.choices.get(0).unwrap().message.clone()
    }

    pub async fn submit(&self, messages: &Vec<Message>) -> ApiResponse {
        eprintln!("SENDING {:?}", messages);

        self.request(messages)
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json::<ApiResponse>()
            .await
            .unwrap()
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
        };

        let json = serde_json::to_string(&message).unwrap();

        assert_eq!(json, r#"{"role":"user","content":"Hello, world!"}"#);
    }
}
