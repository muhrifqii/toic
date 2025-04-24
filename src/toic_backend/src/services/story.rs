use std::sync::Arc;

use candid::Principal;
use lazy_static::lazy_static;

use crate::{
    repositories::story::{
        StoryContentRepository, StoryRepository, STORY_CONTENT_REPOSITORY, STORY_REPOSITORY,
    },
    structure::BinaryTreeRepository,
    types::{ServiceError, ServiceResult, Story},
};

lazy_static! {
    pub static ref STORY_SERVICE: Arc<StoryService> = Arc::new(StoryService::new(
        STORY_REPOSITORY.clone(),
        STORY_CONTENT_REPOSITORY.clone()
    ));
}

#[derive(Debug, Default)]
pub struct StoryService {
    story_repository: Arc<StoryRepository>,
    story_content_repository: Arc<StoryContentRepository>,
}

impl StoryService {
    pub fn new(
        story_repository: Arc<StoryRepository>,
        story_content_repository: Arc<StoryContentRepository>,
    ) -> Self {
        Self {
            story_repository,
            story_content_repository,
        }
    }

    // pub fn get_suggested_stories(
    //     &self,
    //     identity: Principal,
    //     cursor: Option<u64>,
    //     limit: usize,
    // ) -> ServiceResult<(Option<u64>, Vec<Story>)> {
    // }

    pub fn get_story(&self, id: &u64) -> ServiceResult<Story> {
        self.story_repository
            .get(id)
            .ok_or(ServiceError::StoryNotFound)
    }
}
