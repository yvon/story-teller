use crate::narrator::Story;
use std::io::{stdout, Write};
use tokio::io::{self, AsyncBufReadExt, BufReader};

pub async fn read_choice() -> String {
    let mut reader = BufReader::new(io::stdin());
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .expect("failed to read line");
    line
}

pub fn display_story(story: &Story) {
    let mut lock = stdout().lock();

    println!("\n-----\n{}\n", story.text);
    for (i, choice) in story.choices.iter().enumerate() {
        println!("  {}: {}", i + 1, choice);
    }

    print!("\n> ");
    lock.flush().unwrap();
}
