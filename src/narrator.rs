use serde::Deserialize;

const INITIAL_PROMPT_PATH: &str = "initial_prompt.txt";

#[derive(Deserialize)]
pub struct Story {
    pub text: String,
    pub choices: Vec<String>,
}

pub struct BasicNarrator {
    initial_prompt: String,
}

impl BasicNarrator {
    pub fn new() -> Self {
        Self {
            initial_prompt: initial_prompt(),
        }
    }

    pub fn initial_prompt(&self) -> &String {
        &self.initial_prompt
    }
}

pub fn parse_chat_message(message: &String) -> Result<Story, serde_json::Error> {
    serde_json::from_str(message)
}

fn initial_prompt() -> String {
    std::fs::read_to_string(INITIAL_PROMPT_PATH).expect(&format!(
        "Failed to read initial prompt from {}",
        INITIAL_PROMPT_PATH
    ))
}
