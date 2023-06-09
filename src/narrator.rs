use crate::chat::{Message, Role, Service};
use serde::{self, Deserialize};
use std::sync::Arc;
use tokio::task::JoinHandle;

const TOKEN_THRESHOLD_FOR_SUMMARY: u32 = 1000;
const TOKEN_THRESHOLD_FOR_REDUCE: u32 = 1500;

#[derive(Deserialize)]
struct ChatResponse {
    pub text: String,
    pub choices: Vec<String>,
}

#[derive(Deserialize)]
struct SummaryResponse {
    pub summary: String,
}

fn user_message(content: String) -> Message {
    Message {
        role: Role::User,
        content,
    }
}

async fn submit(service: &Service, messages: &Vec<Message>) -> (Message, u32) {
    let api_response = service.submit(messages).await;
    let response_message = api_response.choices.get(0).unwrap().message.clone();
    let total_tokens = api_response.usage.total_tokens;

    eprintln!("Total tokens: {}", total_tokens);
    (response_message, total_tokens)
}

async fn summarize(service: &Service, mut messages: Vec<Message>) -> (usize, String) {
    messages.push(Message {
        role: Role::User,
        content: include_str!("summarize.txt").to_string(),
    });

    let response_message = service.submit_and_return_message(&messages).await;
    let json_response: SummaryResponse = serde_json::from_str(&response_message.content).unwrap();

    eprintln!("SUMMARY: {}", json_response.summary);
    (messages.len(), json_response.summary)
}

fn parse_response(message: &Message) -> ChatResponse {
    serde_json::from_str(&message.content).unwrap()
}

type Parent = Option<Arc<Chapter>>;

pub struct Chapter {
    text: String,
    query: Message,
    response: Message,
    total_tokens: u32,
    choices: Vec<String>,
    parent: Parent,
}

fn collect_messages(parent: &Parent) -> Vec<Message> {
    if parent.is_none() {
        return Vec::new();
    }

    let mut current_chapter = parent.as_ref().unwrap();
    let mut messages = Vec::new();

    while let Some(parent_chapter) = current_chapter.parent.as_ref() {
        messages.push(current_chapter.response.clone());
        messages.push(current_chapter.query.clone());
        current_chapter = parent_chapter;
    }

    messages.push(current_chapter.response.clone());
    messages.push(current_chapter.query.clone());
    messages.reverse();
    messages
}

impl Chapter {
    async fn load(service: &Service, parent: Parent, content: String) -> Self {
        let mut messages = collect_messages(&parent);
        let query = user_message(content);

        messages.push(query.clone());

        let (response, total_tokens) = submit(&service, &messages).await;
        let parsed_response = parse_response(&response);

        Self {
            total_tokens,
            parent,
            query,
            response,
            text: parsed_response.text,
            choices: parsed_response.choices,
        }
    }
}

pub struct Story {
    service: Service,
    current_chapter: Arc<Chapter>,
    next_chapters: Vec<JoinHandle<Chapter>>,
}

impl Story {
    pub async fn new(service: Service) -> Self {
        let content = include_str!("initial_prompt.txt").to_string();
        let current_chapter = Arc::new(Chapter::load(&service, None, content).await);
        let next_chapters = next_chapters(&service, &current_chapter);

        Self {
            service,
            current_chapter,
            next_chapters,
        }
    }

    pub fn text(&self) -> &String {
        &self.current_chapter.as_ref().text
    }

    pub fn choices(&self) -> &Vec<String> {
        &self.current_chapter.as_ref().choices
    }

    pub fn loaded(&self, index: usize) -> bool {
        self.next_chapters[index].is_finished()
    }

    pub async fn choose(&mut self, index: usize) {
        let chapter = self.next_chapters.swap_remove(index).await.unwrap();
        self.current_chapter = Arc::new(chapter);
        self.next_chapters = next_chapters(&self.service, &self.current_chapter);
    }
}

fn next_chapters(service: &Service, current_chapter: &Arc<Chapter>) -> Vec<JoinHandle<Chapter>> {
    current_chapter
        .as_ref()
        .choices
        .iter()
        .map(|choice| {
            let service = service.clone();
            let content = choice.clone();
            let parent = Some(current_chapter.clone());

            tokio::task::spawn(async move { Chapter::load(&service, parent, content).await })
        })
        .collect()
}
