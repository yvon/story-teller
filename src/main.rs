mod chat;
mod interraction;
mod narrator;

use narrator::BasicNarrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let narrator = BasicNarrator::new();
    let mut chat_service = chat::Service::new();

    let mut prompt: String = narrator.initial_prompt().clone();

    loop {
        let resp = chat_service.query(&prompt).await.unwrap();
        let story = narrator::parse_chat_message(&resp).unwrap();
        interraction::display_story(&story);

        prompt = interraction::read_choice();
    }
}
