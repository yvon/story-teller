mod chat;
mod interraction;
mod narrator;

use chat::{Message, Role};
use narrator::BasicNarrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let narrator = BasicNarrator::new();
    let chat_service = chat::Service::new();

    let mut message = Message {
        role: Role::User,
        content: narrator.initial_prompt().clone(),
    };

    let mut messages = vec![message.clone()];

    loop {
        let resp = chat_service.submit(&messages).await.unwrap();
        let story = narrator::parse_chat_message(&resp.content).unwrap();
        interraction::display_story(&story);

        messages.push(resp);
        eprintln!(
            "SUMMARIZE: {:?}",
            narrator::summarize(&chat_service, &messages).await
        );
        message.content = interraction::read_choice();
    }
}
