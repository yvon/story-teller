use crate::chat::Service;
use chapter::Chapter;
use std::sync::Arc;
use tokio::task::JoinHandle;

mod chapter;
mod messages;
mod summarize;

pub struct Story {
    service: Service,
    current_chapter: Arc<Chapter>,
    next_chapters: Vec<JoinHandle<Chapter>>,
}

impl Story {
    pub async fn new(service: Service) -> Self {
        let content = include_str!("initial_prompt.txt").to_string();
        let chapter = Chapter::load(&service, None, content).await;

        let mut story = Self {
            service,
            current_chapter: Arc::new(chapter),
            next_chapters: Vec::new(),
        };

        story.preload_next_chapters();
        story
    }

    pub fn chapter(&self) -> (&String, &Vec<String>) {
        let current_chapter = self.current_chapter.as_ref();
        (current_chapter.text(), &current_chapter.choices())
    }

    pub fn loaded(&self, index: usize) -> bool {
        self.next_chapters[index].is_finished()
    }

    pub async fn choose(&mut self, index: usize) {
        let chapter = self.next_chapters.swap_remove(index).await.unwrap();
        self.current_chapter = Arc::new(chapter);
        self.preload_next_chapters();
    }

    fn preload_next_chapters(&mut self) {
        self.next_chapters = self
            .current_chapter
            .as_ref()
            .choices()
            .iter()
            .map(|choice| {
                let service = self.service.clone();
                let content = choice.clone();
                let parent = Some(self.current_chapter.clone());

                tokio::task::spawn(async move { Chapter::load(&service, parent, content).await })
            })
            .collect()
    }
}
