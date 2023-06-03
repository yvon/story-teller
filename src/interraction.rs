use crate::narrator::Story;
use std::io::{stdin, stdout, BufRead, Write};

pub fn read_choice() -> String {
    let mut line = String::new();
    stdin()
        .lock()
        .read_line(&mut line)
        .expect("Failed to read line");
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
