use futures::future::{select, Either};
use futures::Future;
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

pub async fn read_choice(future: impl Future<Output = usize>) -> usize {
    let read_line_future = Box::pin(read_line());
    let future = Box::pin(future);

    match select(read_line_future, future).await {
        Either::Left((line, pending)) => {
            println!("Still fetching story, please wait...");
            pending.await;
        }
        Either::Right((a, b)) => {
            eprintln!("Stories loaded");
            b.await;
        }
    };

    1
    //  loop {
    //      tokio::select! {

    //          },
    //          cn_choices = &mut future => {
    //              if let Some(valid_choice) = valid_choice(&line, &cn_choices) {
    //                  return valid_choice;
    //              } else {
    //                  println!("Invalid choice, please try again.");
    //                  line.clear();
    //              }
    //          }
    //      }
    //  }
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
