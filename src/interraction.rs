use std::io::{self, BufRead};

pub fn read_answer() -> String {
    let mut line = String::new();
    io::stdin()
        .lock()
        .read_line(&mut line)
        .expect("Failed to read line");
    line
}
