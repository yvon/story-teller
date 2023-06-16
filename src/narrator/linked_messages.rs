use crate::chat::Message;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct SharedMessage(Arc<RwLock<LinkedMessage>>);

#[derive(Clone, Debug)]
pub struct LinkedMessage {
    pub message: Message,
    pub parent: Option<SharedMessage>,
    pub total_tokens: Option<u32>,
}

impl SharedMessage {
    pub fn new(message: Message, parent: Option<SharedMessage>, total_tokens: Option<u32>) -> Self {
        SharedMessage(Arc::new(RwLock::new(LinkedMessage {
            message,
            parent,
            total_tokens,
        })))
    }

    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, LinkedMessage> {
        self.0.read().unwrap()
    }

    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, LinkedMessage> {
        self.0.write().unwrap()
    }
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
                    linked_message = reference.read().clone();
                }
            }
        }

        messages.reverse();
        messages
    }
}
