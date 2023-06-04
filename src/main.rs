mod chat;
mod interraction;
mod narrator;

use chat::Service;
use narrator::BasicNarrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = Service::new();
    let mut narrator = BasicNarrator::new(service).await;

    loop {
        interraction::display_story(narrator.story());

        // TODO: Remove
        eprintln!("SUMMARIZE: {:?}", narrator.summarize().await);

        narrator.choose(interraction::read_choice()).await;
    }
}
