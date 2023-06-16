use super::{message_above_threshold, Chapter, Summary};
use crate::chat::Service;
use tokio::task::{spawn, JoinHandle};

const TOKEN_THRESHOLD_FOR_REDUCE: u32 = 3500;

pub struct Story {
    service: Service,
    current_chapter: Chapter,
    next_chapters: Vec<JoinHandle<Chapter>>,
    summary: Option<JoinHandle<Summary>>,
}

impl Story {
    pub async fn new(service: Service) -> Self {
        let content = include_str!("initial_prompt.txt").to_string();
        let chapter = Chapter::load(&service, None, content).await;

        let mut story = Self {
            service,
            current_chapter: chapter,
            next_chapters: Vec::new(),
            summary: None,
        };

        story.preload_next_chapters();
        story
    }

    pub fn chapter(&self) -> (&String, &Vec<String>) {
        (self.current_chapter.text(), &self.current_chapter.choices())
    }

    pub fn loaded(&self, index: usize) -> bool {
        self.next_chapters[index].is_finished()
    }

    pub async fn choose(&mut self, index: usize) {
        let chapter = self.next_chapters.swap_remove(index).await.unwrap();

        self.handle_token_thresholds(&chapter).await;
        self.current_chapter = chapter;
        self.preload_next_chapters();
    }

    fn preload_next_chapters(&mut self) {
        self.next_chapters = self
            .current_chapter
            .choices()
            .iter()
            .map(|choice| {
                let service = self.service.clone();
                let content = format!(include_str!("next_chapter.txt"), choice.clone());
                let parent = Some(self.current_chapter.message().clone());

                spawn(async move { Chapter::load(&service, parent, content).await })
            })
            .collect()
    }

    async fn handle_token_thresholds(&mut self, chapter: &Chapter) {
        if self.summary.is_none() {
            if let Some(message) = message_above_threshold(chapter.message().clone()) {
                let service = self.service.clone();
                let join_handle = spawn(Summary::new(service, message));
                self.summary = Some(join_handle);
            }
        }

        if let Some(value) = chapter.message().read().total_tokens {
            if value > TOKEN_THRESHOLD_FOR_REDUCE {
                self.reduce_history().await;
            }
        }
    }

    async fn reduce_history(&mut self) {
        let result = self.summary.take().unwrap().await;
        let summary = result.unwrap();
        let mut message = summary.message.write();

        message.parent = None;
        message.message.content = Some(summary.content);
    }
}
