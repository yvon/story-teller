use crate::chat::{Message, Role};
use message_manager::MessageManager;
use serde::Deserialize;

mod message_manager;

#[derive(Deserialize)]
pub struct Story {
    pub text: String,
    pub choices: Vec<String>,
}

pub struct BasicNarrator {
    message_manager: MessageManager,
    story: Story,
}

impl BasicNarrator {
    pub async fn new() -> Self {
        let mut message_manager = MessageManager::new(initial_message());
        let first_message = message_manager.submit().await;
        let story = parse_story(&first_message);

        Self {
            message_manager,
            story,
        }
    }

    pub fn story(&self) -> &Story {
        &self.story
    }

    pub async fn choose(&mut self, choice: String) {
        let message = Message {
            role: Role::User,
            content: choice,
        };
        let reply = self.message_manager.reply(message).await;
        self.story = parse_story(&reply);
    }

    pub async fn post_processing(&mut self) {
        self.message_manager.post_processing().await;
    }
}

pub fn read_prompt(path: &'static str) -> String {
    std::fs::read_to_string(path).expect(&format!("Failed to read initial prompt from {}", path))
}

fn initial_message() -> Message {
    let initial_prompt = read_prompt("initial_prompt.txt");

    Message {
        role: Role::User,
        content: initial_prompt,
    }
}

fn parse_story(message: &Message) -> Story {
    serde_json::from_str(&message.content).unwrap()
}
