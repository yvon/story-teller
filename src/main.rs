mod chat;
mod interraction;
mod narrator;

use chat::Service;
use interraction::{display_story, read_choice};
use narrator::BasicNarrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = Service::new();
    let mut narrator = BasicNarrator::new(service);

    narrator.start().await;

    loop {
        let story = narrator.story().as_ref().unwrap();
        display_story(story);
        // Read STDIN asynchronously so we can perform post processing operations meanwhile.
        let (choice, _) = tokio::join!(read_choice(), narrator.post_processing());
        narrator.choose(choice).await;
    }
}
