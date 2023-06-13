use crate::chat::{Message, Service};
use chapter::Chapter;
use std::sync::Arc;
use summarize::SummarizedChapter;
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
    summarized_chapter: Option<JoinHandle<SummarizedChapter>>,
}

impl Story {
    pub async fn new(service: Service) -> Self {
        let content = include_str!("initial_prompt.txt").to_string();
        let chapter = Chapter::load(&service, None, content).await;

        let mut story = Self {
            service,
            current_chapter: chapter,
            next_chapters: Vec::new(),
            summarized_chapter: None,
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

        // self.handle_token_thresholds(&chapter);
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
                let parent = Some(self.current_chapter.message());

                spawn(async move { Chapter::load(&service, parent, content).await })
            })
            .collect()
    }

    // fn handle_token_thresholds(&mut self, chapter: &Chapter) {
    //     let tokens = chapter.total_tokens();

    //     if tokens > TOKEN_THRESHOLD_FOR_SUMMARY {
    //         self.initiate_summary_creation();
    //     } else if tokens > TOKEN_THRESHOLD_FOR_REDUCE {
    //         self.reduce_history();
    //     }
    // }

    // fn initiate_summary_creation(&mut self) {
    //     let service = self.service.clone();
    //     let chapter = self.current_chapter.clone();
    //     let join_handle = spawn(SummarizedChapter::new(service, chapter));

    //     self.summarized_chapter = Some(join_handle);
    // }

    // async fn reduce_history(&mut self) {
    //     let summarized_chapter = &self.summarized_chapter;

    //     let mut a = summarized_chapter.unwrap().await.unwrap().chapter.as_ref();
    //     a.parent = None;
    // }
}
