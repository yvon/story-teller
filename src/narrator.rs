const INITIAL_PROMPT_PATH: &str = "initial_prompt.txt";

pub trait Narrator {
    fn initial_prompt(&self) -> &String;
}

pub struct BasicNarrator {
    initial_prompt: String,
}

impl BasicNarrator {
    pub fn new() -> Self {
        Self {
            initial_prompt: yolo_initial_prompt(),
        }
    }
}

impl Narrator for BasicNarrator {
    fn initial_prompt(&self) -> &String {
        &self.initial_prompt
    }
}

fn yolo_initial_prompt() -> String {
    std::fs::read_to_string(INITIAL_PROMPT_PATH).expect(&format!(
        "Failed to read initial prompt from {}",
        INITIAL_PROMPT_PATH
    ))
}
