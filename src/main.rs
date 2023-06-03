mod chat;
mod interraction;
mod narrator;

use narrator::{BasicNarrator, Narrator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let narrator = BasicNarrator::new();
    let mut chat_service = chat::Service::new();

    let resp = chat_service
        .query(&narrator.initial_prompt())
        .await
        .unwrap();

    println!("{}", resp);

    loop {
        let answer = interraction::read_answer();
        let resp = chat_service.query(&answer).await?;
        println!("{}", resp);
    }
}
