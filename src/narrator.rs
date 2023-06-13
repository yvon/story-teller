use crate::chat::Service;
use chapter::Chapter;
use summarize::Summary;
use tokio::task::{spawn, JoinHandle};

mod chapter;
mod messages;
mod summarize;

const TOKEN_THRESHOLD_FOR_SUMMARY: u32 = 1000;
const TOKEN_THRESHOLD_FOR_REDUCE: u32 = 1500;

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
                let content = choice.clone();
                let parent = Some(self.current_chapter.message().clone());

                spawn(async move { Chapter::load(&service, parent, content).await })
            })
            .collect()
    }

    async fn handle_token_thresholds(&mut self, chapter: &Chapter) {
        let tokens = chapter.total_tokens();

        if tokens > TOKEN_THRESHOLD_FOR_SUMMARY {
            self.initiate_summary_creation();
        } else if tokens > TOKEN_THRESHOLD_FOR_REDUCE {
            self.reduce_history().await;
        }
    }

    fn initiate_summary_creation(&mut self) {
        let service = self.service.clone();
        let message = self.current_chapter.message().clone();
        let join_handle = spawn(Summary::new(service, message));

        self.summary = Some(join_handle);
    }

    async fn reduce_history(&mut self) {
        let result = self.summary.take().unwrap().await;
        let summary = result.unwrap();
        let lock = summary.message.as_ref();
        let mut message = lock.write().unwrap();

        message.parent = None;
        message.content = summary.content;
    }
}
