mod chat;
mod interraction;
mod narrator;

#[tokio::main]
async fn main() {
    let service = chat::Service::new();
    let story = narrator::Story::new(service).await;

    interraction::start(story).await;
}
