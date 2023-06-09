use std::io::{stdout, Write};
use tokio::io::{self, AsyncBufReadExt, BufReader};

fn valid_choice(choice: &String, cn_choices: &usize) -> Option<usize> {
    let choice_num = choice.trim().parse::<usize>();
    match choice_num {
        Ok(num) if num > 0 && num <= *cn_choices => Some(num - 1),
        _ => None,
    }
}

async fn read_line() -> String {
    let mut line = String::new();
    let mut reader = BufReader::new(io::stdin());
    reader.read_line(&mut line).await.unwrap();
    line
}

pub async fn read_choice(cn_choices: usize) -> usize {
    loop {
        let choice = read_line().await;
        match valid_choice(&choice, &cn_choices) {
            None => {
                println!("Invalid choice");
                continue;
            }
            Some(index) => {
                return index;
            }
        }
    }
}

pub fn display(text: &String, choices: &Vec<String>) {
    let mut lock = stdout().lock();

    println!("\n-----\n{}\n", text);
    for (i, choice) in choices.iter().enumerate() {
        println!("  {}: {}", i + 1, choice);
    }

    print!("\n> ");
    lock.flush().unwrap();
}
