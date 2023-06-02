use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Message {
    pub role: Role,
    pub content: String,
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

#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    pub choices: Vec<Choice>,
}

pub struct Service {
    client: reqwest::Client,
    api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
}

impl Service {
    pub fn new() -> Self {
        let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    pub async fn request(&self, messages: &[Message]) -> reqwest::Result<reqwest::Response> {
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

    pub async fn request_and_parse_response(
        &self,
        messages: &[Message],
    ) -> reqwest::Result<String> {
        let parsed_json = self
            .request(messages)
            .await?
            .error_for_status()?
            .json::<ApiResponse>()
            .await?;

        let choice = parsed_json.choices.get(0).unwrap();

        Ok(choice.message.content.clone())
    }
}
