mod chat;
mod narrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chat_service = chat::Service::new();
    let initial_prompt = narrator::initial_prompt();

    let message = chat::Message {
        role: chat::Role::User,
        content: initial_prompt,
    };

    let resp = chat_service.request_and_parse_response(&[message]).await?;

    println!("{:#?}", resp);
    Ok(())
}
