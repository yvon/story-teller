mod chat;
mod interraction;
mod narrator;

use chat::Service;
use interraction::{display, read_choice};
use narrator::BasicNarrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = Service::new();
    let mut narrator = BasicNarrator::new(service).await;

    loop {
        display(narrator.story(), narrator.choices());
        let choice = read_choice(narrator.post_processing()).await;
        narrator.choose(choice);
    }
}
