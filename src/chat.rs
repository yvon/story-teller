use serde::{Deserialize, Serialize};
use std::env;

pub mod request;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
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

    pub async fn submit(&self, body: request::Body) -> ApiResponse {
        let response = self.request(body).await.unwrap();
        let status = response.status();
        let text = response.text().await.unwrap();

        if status.is_client_error() || status.is_server_error() {
            panic!("Error: {}", text);
        }

        let api_response: ApiResponse = serde_json::from_str(&text).unwrap();
        eprintln!("RECEIVED {:#?}", api_response);
        api_response
    }

    async fn request(&self, body: request::Body) -> reqwest::Result<reqwest::Response> {
        let json = serde_json::to_string_pretty(&body).unwrap();
        eprintln!("SENDING {}", &json);

        self.client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
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

    pub fn user_message() -> Message {
        Message {
            role: Role::User,
            content: Some(String::from("Hello, world!")),
            function_call: None,
            name: None,
        }
    }

    #[test]
    fn test_message_deserialization() {
        let json = r#"{
            "role": "user",
            "content": "Hello, world!"
        }"#;

        let message: Message = serde_json::from_str(json).unwrap();

        assert_eq!(message.role, Role::User);
        assert_eq!(message.content, Some(String::from("Hello, world!")));
    }

    #[test]
    fn test_message_serialization() {
        let message = user_message();
        let json = serde_json::to_string(&message).unwrap();

        assert_eq!(json, r#"{"role":"user","content":"Hello, world!"}"#);
    }
}
