use std::sync::Arc;

use crate::{repositories::story::StoryRepository, types::Story};

#[derive(Debug, Default)]
pub struct StoryService {
    story_repository: Arc<StoryRepository>,
}

impl StoryService {
    pub fn new(story_repository: Arc<StoryRepository>) -> Self {
        Self { story_repository }
    }
}
