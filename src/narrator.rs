use crate::chat::{Message, Role, Service};
use serde::{self, Deserialize};
use std::rc::Rc;
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

pub async fn submit(service: &Service, messages: &Vec<Message>) -> (Message, u32) {
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

type Parent = Option<Rc<Node>>;

struct Node {
    story: String,
    query: Message,
    response: Message,
    choices: Vec<String>,
    parent: Parent,
}

fn collect_messages(node: &Node) -> Vec<Message> {
    let mut messages = Vec::new();
    let mut current_node = node;

    // Loop until the parent of current node is None
    while let Some(parent_node) = current_node.parent.as_ref() {
        // Add the response and query to messages
        messages.push(current_node.response.clone());
        messages.push(current_node.query.clone());

        // Traverse to parent
        current_node = &**parent_node;
    }

    // Don't forget to push the response and query of the root node
    messages.push(current_node.response.clone());
    messages.push(current_node.query.clone());

    // Reverse the messages to maintain the chronological order
    messages.reverse();
    messages
}

impl Node {
    async fn new(service: &Service, content: String, parent: Parent) -> Self {
        let mut messages = match &parent {
            Some(node) => collect_messages(node.as_ref()),
            None => Vec::new(),
        };

        let query = Message {
            role: Role::User,
            content,
        };

        messages.push(query.clone());

        let (response, _) = submit(&service, &messages).await;
        let parsed_response = parse_response(&response);

        Self {
            parent,
            query,
            response,
            story: parsed_response.text,
            choices: parsed_response.choices,
        }
    }
}

pub struct BasicNarrator {
    service: Service,
    current_node: Rc<Node>,
    children_handles: Option<Vec<JoinHandle<Node>>>,
}

impl BasicNarrator {
    pub async fn new(service: Service) -> Self {
        let content = include_str!("initial_prompt.txt").to_string();
        let current_node = Node::new(&service, content, None).await;

        Self {
            service,
            current_node: Rc::new(current_node),
            children_handles: None,
        }
    }

    pub fn story(&self) -> &String {
        &self.current_node.as_ref().story
    }

    pub fn choices(&self) -> &Vec<String> {
        &self.current_node.as_ref().choices
    }

    pub async fn post_processing(&mut self) -> usize {
        let children = self.children().await;
        let len = children.len();

        self.children = Some(children);
        len
    }

    pub fn choose(&mut self, index: usize) {
        let chosen_node = self.children.take().unwrap().swap_remove(index);
        self.current_node = Rc::new(chosen_node);
    }

    async fn children(&mut self) {
        self.children_handles = self
            .current_node
            .choices
            .iter()
            .map(|choice| {
                let service = self.service.clone();
                let content = choice.clone();
                let parent = Some(self.current_node.clone());

                tokio::task::spawn(async move { Node::new(&service, content, parent).await })
            })
            .collect();
    }
}
