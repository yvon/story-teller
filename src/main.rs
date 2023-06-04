mod chat;
mod interraction;
mod narrator;

use interraction::{display_story, read_choice};
use narrator::BasicNarrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut narrator = BasicNarrator::new().await;

    loop {
        display_story(narrator.story());
        // Read STDIN asynchronously so we can perform post processing operations meanwhile.
        let (choice, _) = tokio::join!(read_choice(), narrator.post_processing());
        narrator.choose(choice).await;
    }
}
