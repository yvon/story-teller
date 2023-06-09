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
        let chapter = read_choice(story.children().await).await;
        story.choose(chapter);
    }
}
