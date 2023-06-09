mod chat;
mod interraction;
mod narrator;

use chat::Service;
use interraction::{display, read_choice};
use narrator::Story;

#[tokio::main]
async fn main() {
    let service = Service::new();
    let mut story = Story::new(service).await;

    loop {
        display(story.text(), story.choices());
        let index = read_choice(story.choices().len()).await;
        if !story.loaded(index) {
            println!("Loading...");
        }
        story.choose(index).await;
    }
}
