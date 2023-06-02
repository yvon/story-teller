const INITIAL_PROMPT_PATH: &str = "initial_prompt.txt";

pub fn initial_prompt() -> String {
    std::fs::read_to_string(INITIAL_PROMPT_PATH).expect(&format!(
        "Failed to read initial prompt from {}",
        INITIAL_PROMPT_PATH
    ))
}
