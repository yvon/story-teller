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

#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    pub choices: Vec<Choice>,
}

pub struct Service {
    client: reqwest::Client,
    api_key: String,
    messages: Vec<Message>,
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
            messages: Vec::new(),
            api_key,
        }
    }

    pub async fn query(&mut self, content: &String) -> reqwest::Result<String> {
        self.messages.push(Message {
            role: Role::User,
            content: content.clone(),
        });

        self.submit().await
    }

    pub async fn submit(&mut self) -> reqwest::Result<String> {
        let parsed_json = self
            .request()
            .await?
            .error_for_status()?
            .json::<ApiResponse>()
            .await?;

        let choice = parsed_json.choices.get(0).unwrap();
        let content = &choice.message.content;

        self.messages.push(Message {
            role: Role::Assistant,
            content: content.clone(),
        });

        Ok(content.clone())
    }

    async fn request(&self) -> reqwest::Result<reqwest::Response> {
        self.client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({
                "model": "gpt-3.5-turbo",
                "messages": &self.messages
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
