use std::io::{self, BufRead};

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

    let mut messages = Vec::new();
    messages.push(message); // TODO: initialize with a vec macro

    let resp = chat_service.request_and_parse_response(&messages).await?;

    println!("{:#?}", resp);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(line) => {
                messages.push(chat::Message {
                    role: chat::Role::User,
                    content: line,
                });
                let resp = chat_service.request_and_parse_response(&messages).await?;
                println!("{:#?}", resp);
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                break;
            }
        }
    }

    Ok(())
}
