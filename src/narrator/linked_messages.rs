use crate::chat::Message;
use std::sync::{Arc, RwLock};

pub type SharedMessage = Arc<RwLock<LinkedMessage>>;

#[derive(Clone)]
pub struct LinkedMessage {
    pub message: Message,
    pub parent: Option<SharedMessage>,
}

impl LinkedMessage {
    pub fn messages(&self) -> Vec<Message> {
        let mut messages: Vec<Message> = Vec::new();
        let mut linked_message = self.clone();

        loop {
            let parent = linked_message.parent;
            let message = linked_message.message;

            messages.push(message);

            match parent {
                None => break,
                Some(reference) => {
                    linked_message = reference.as_ref().read().unwrap().clone();
                }
            }
        }

        messages.reverse();
        messages
    }
}
