use std::sync::Arc;

use candid::Principal;
use lazy_static::lazy_static;

use crate::{
    repositories::{
        draft::{DraftRepository, DRAFT_REPOSITORY},
        story::{StoryRepository, STORY_REPOSITORY},
    },
    structure::{AuditableRepository, BinaryTreeRepository},
    types::{Draft, RepositoryError, ServiceError, ServiceResult, Story, StoryLabel},
};

lazy_static! {
    pub static ref DRAFT_SERVICE: Arc<DraftService> = Arc::new(DraftService::new(
        DRAFT_REPOSITORY.clone(),
        STORY_REPOSITORY.clone()
    ));
}

#[derive(Debug, Default)]
pub struct DraftService {
    draft_repository: Arc<DraftRepository>,
    story_repository: Arc<StoryRepository>,
}

impl DraftService {
    pub fn new(
        draft_repository: Arc<DraftRepository>,
        story_repository: Arc<StoryRepository>,
    ) -> Self {
        Self {
            draft_repository,
            story_repository,
        }
    }

    pub fn create_draft(
        &self,
        title: String,
        content: String,
        author: Principal,
        ai_used: bool,
    ) -> ServiceResult<Draft> {
        let draft = Draft::new(title, content, author, ai_used);

        self.draft_repository
            .insert(draft)
            .map_err(|e| ServiceError::InternalError {
                reason: format!("Failed to create draft: {}", e),
            })
    }

    pub fn update_draft(
        &self,
        id: u64,
        new_title: Option<String>,
        new_content: Option<String>,
        identity: Principal,
    ) -> ServiceResult<Draft> {
        if new_title.is_none() && new_content.is_none() {
            return Err(ServiceError::UnprocessableEntity {
                reason: "No fields to update".to_string(),
            });
        }

        let mut draft = self
            .draft_repository
            .get(&id)
            .ok_or(ServiceError::DraftNotFound)?;
        validate_draft_author(&draft, identity)?;

        if let Some(new_title) = new_title {
            draft.title = new_title;
        }
        if let Some(new_content) = new_content {
            draft.content = new_content;
        }

        self.draft_repository.update(draft).map_err(|e| match e {
            RepositoryError::NotFound => ServiceError::DraftNotFound,
            _ => ServiceError::InternalError {
                reason: format!("Failed to update draft: {}", e),
            },
        })
    }

    pub fn publish_draft(&self, id: u64, identity: Principal) -> ServiceResult<Story> {
        let draft = self
            .draft_repository
            .get(&id)
            .ok_or(ServiceError::DraftNotFound)?;
        validate_draft_author(&draft, identity)?;

        ic_cdk::println!("Publishing draft: {:?} by {:?}", &draft, identity);

        // Promote to story
        let story = Story::new(
            draft.title.clone(),
            draft.content.clone(),
            if draft.ai_used {
                StoryLabel::AI
            } else {
                StoryLabel::OC
            },
            draft.author,
        );

        let inserted_story =
            self.story_repository
                .insert(story)
                .map_err(|e| ServiceError::IdentityNotFound {
                    identity: e.to_string(),
                })?;

        self.draft_repository.delete(&id).map_err(|e| match e {
            RepositoryError::NotFound => ServiceError::DraftNotFound,
            _ => ServiceError::InternalError {
                reason: format!("Failed to delete draft: {}", e),
            },
        })?;

        Ok(inserted_story)
    }

    pub fn delete_draft(&self, id: u64, identity: Principal) -> ServiceResult<u64> {
        let draft = self
            .draft_repository
            .get(&id)
            .ok_or(ServiceError::DraftNotFound)?;
        validate_draft_author(&draft, identity)?;

        self.draft_repository.delete(&id).map_err(|e| match e {
            RepositoryError::NotFound => ServiceError::DraftNotFound,
            _ => ServiceError::InternalError {
                reason: format!("Failed to delete draft: {}", e),
            },
        })
    }

    pub fn get_draft(&self, id: &u64) -> ServiceResult<Draft> {
        self.draft_repository
            .get(id)
            .ok_or(ServiceError::DraftNotFound)
    }

    pub fn get_drafts(
        &self,
        identity: Principal,
        cursor: Option<u64>,
        limit: usize,
    ) -> ServiceResult<(Option<u64>, Vec<Draft>)> {
        let drafts = self
            .draft_repository
            .get_drafts_by_author(identity, cursor, limit)
            .map_err(|e| ServiceError::InternalError {
                reason: format!("Failed to get drafts: {}", e),
            })?;

        Ok((drafts.last().map(|d| d.id), drafts))
    }
}

fn validate_draft_author(draft: &Draft, identity: Principal) -> Result<(), ServiceError> {
    if draft.author != identity {
        return Err(ServiceError::IdentityUnauthorized {
            identity: identity.to_string(),
        });
    }
    Ok(())
}
