use std::sync::Arc;

use candid::Principal;
use lazy_static::lazy_static;

use crate::{
    repositories::{
        draft::{
            self, DraftContentRepository, DraftRepository, DRAFT_CONTENT_REPOSITORY,
            DRAFT_REPOSITORY,
        },
        story::{
            StoryContentRepository, StoryRepository, STORY_CONTENT_REPOSITORY, STORY_REPOSITORY,
        },
    },
    structure::{AuditableRepository, BinaryTreeRepository, Repository},
    types::{
        Draft, RepositoryError, SaveDraftArgs, ServiceError, ServiceResult, Story, StoryContent,
    },
    utils::estimate_read_time,
};

lazy_static! {
    pub static ref DRAFT_SERVICE: Arc<DraftService> = Arc::new(DraftService::new(
        DRAFT_REPOSITORY.clone(),
        DRAFT_CONTENT_REPOSITORY.clone(),
        STORY_REPOSITORY.clone(),
        STORY_CONTENT_REPOSITORY.clone(),
    ));
}

#[derive(Debug, Default)]
pub struct DraftService {
    draft_repository: Arc<DraftRepository>,
    draft_content_repository: Arc<DraftContentRepository>,
    story_repository: Arc<StoryRepository>,
    story_content_repository: Arc<StoryContentRepository>,
}

impl DraftService {
    pub fn new(
        draft_repository: Arc<DraftRepository>,
        draft_content_repository: Arc<DraftContentRepository>,
        story_repository: Arc<StoryRepository>,
        story_content_repository: Arc<StoryContentRepository>,
    ) -> Self {
        Self {
            draft_repository,
            draft_content_repository,
            story_repository,
            story_content_repository,
        }
    }

    pub async fn create_draft(
        &self,
        args: SaveDraftArgs,
        identity: Principal,
    ) -> ServiceResult<Draft> {
        validate_empty_save_args(&args, "Nothing to save")?;

        let draft = Draft::new(args.title.unwrap_or_default(), args.detail, identity);
        let content = args.content.unwrap_or_default();

        let draft = self.draft_repository.insert(draft).map_err(|e| match e {
            RepositoryError::Conflict => ServiceError::Conflict {
                entity: "Draft".to_string(),
            },
            _ => ServiceError::InternalError {
                reason: format!("Failed to create draft: {}", e),
            },
        })?;
        let content = self
            .draft_content_repository
            .insert(StoryContent::new(draft.id, content, identity))
            .map_err(|e| match e {
                _ => ServiceError::InternalError {
                    reason: format!("Failed to create draft content: {}", e),
                },
            });
        if let Some(content_err) = &content.err() {
            // rollback draft creation
            self.draft_repository
                .delete(&draft.id)
                .map_err(|e| ServiceError::InternalError {
                    reason: format!(
                        "Failed to rollback draft creation: {}. original error: {}",
                        e, content_err
                    ),
                })?;
            // successfully rollback, and report the error
            return Err(content_err.clone());
        }
        Ok(draft)
    }

    pub async fn update_draft(
        &self,
        id: u64,
        args: SaveDraftArgs,
        identity: Principal,
    ) -> ServiceResult<()> {
        validate_empty_save_args(&args, "Nothing to update")?;

        let mut draft = self
            .draft_repository
            .get(&id)
            .ok_or(ServiceError::DraftNotFound)?;
        validate_draft_author(draft.author, identity)?;

        if let Some(new_title) = &args.title {
            draft.title = new_title.to_string();
        }
        let is_detail_changed = args.detail.is_some();
        if is_detail_changed {
            draft.detail = args.detail;
        }

        if let Some(new_content) = args.content {
            let mut d_content = self
                .draft_content_repository
                .get(&id)
                .ok_or(ServiceError::DraftNotFound)?;
            validate_draft_author(d_content.author, identity)?;

            let new_read_estimate = estimate_read_time(&new_content);
            let read_estimate_diff = new_read_estimate.abs_diff(draft.read_time);
            d_content.content = new_content;
            self.draft_content_repository
                .update(d_content)
                .map_err(|e| match e {
                    RepositoryError::NotFound => ServiceError::DraftNotFound,
                    _ => ServiceError::InternalError {
                        reason: format!("Failed to update draft content: {}", e),
                    },
                })?;
            if read_estimate_diff == 0 && args.title.is_none() && !is_detail_changed {
                // No changes to the draft entity while no significant difference in read time
                // updating content without updating draft entity's read estimation
                return Ok(());
            }
            draft.read_time = new_read_estimate;
        }

        self.draft_repository.update(draft).map_err(|e| match e {
            RepositoryError::NotFound => ServiceError::DraftNotFound,
            _ => ServiceError::InternalError {
                reason: format!("Failed to update draft: {}", e),
            },
        })?;
        Ok(())
    }

    pub async fn publish_draft(&self, id: u64, identity: Principal) -> ServiceResult<Story> {
        let draft = self
            .draft_repository
            .get(&id)
            .ok_or(ServiceError::DraftNotFound)?;
        let d_content = self
            .draft_content_repository
            .get(&id)
            .ok_or(ServiceError::DraftNotFound)?;
        validate_draft_author(draft.author, identity)?;
        validate_draft_author(d_content.author, identity)?;
        if draft.title.is_empty() || d_content.content.is_empty() {
            return Err(ServiceError::UnprocessableEntity {
                reason: "Title and content cannot be empty".to_string(),
            });
        }
        let detail = draft
            .detail
            .clone()
            .filter(|detail| !detail.description.is_empty())
            .ok_or_else(|| ServiceError::UnprocessableEntity {
                reason: "Story detail is required".to_string(),
            })?;

        // Promote to story
        let story = Story::new(draft, detail);
        let story = self.story_repository.insert(story).map_err(|e| match e {
            RepositoryError::Conflict => ServiceError::Conflict {
                entity: "Story".to_string(),
            },
            _ => ServiceError::InternalError {
                reason: format!("Failed to publish draft: {}", e),
            },
        })?;

        let s_content = StoryContent::new(story.id, d_content.content, identity);
        let s_content = self
            .story_content_repository
            .insert(s_content)
            .map_err(|e| ServiceError::InternalError {
                reason: format!("Failed to publish draft content: {}", e),
            });
        if let Some(content_err) = &s_content.err() {
            // rollback publish draft
            self.story_repository
                .delete(&story.id)
                .map_err(|e| ServiceError::InternalError {
                    reason: format!(
                        "Failed to rollback publish draft: {}. original error: {}",
                        e, content_err
                    ),
                })?;
            // successfully rollback, and report the error
            return Err(content_err.clone());
        }

        self.draft_content_repository
            .delete(&id)
            .map_err(|e| ServiceError::InternalError {
                reason: format!("Failed to delete draft content: {}", e),
            })?;
        self.draft_repository.delete(&id).map_err(|e| match e {
            RepositoryError::NotFound => ServiceError::DraftNotFound,
            _ => ServiceError::InternalError {
                reason: format!("Failed to delete draft: {}", e),
            },
        })?;

        Ok(story)
    }

    pub async fn delete_draft(&self, id: u64, identity: Principal) -> ServiceResult<u64> {
        let draft = self
            .draft_repository
            .get(&id)
            .ok_or(ServiceError::DraftNotFound)?;
        validate_draft_author(draft.author, identity)?;

        self.draft_content_repository
            .delete(&id)
            .map_err(|e| ServiceError::InternalError {
                reason: format!("Failed to delete draft content: {}", e),
            })?;
        self.draft_repository.delete(&id).map_err(|e| match e {
            RepositoryError::NotFound => ServiceError::DraftNotFound,
            _ => ServiceError::InternalError {
                reason: format!("Failed to delete draft: {}", e),
            },
        })
    }

    pub fn get_draft(&self, id: &u64) -> ServiceResult<(Draft, StoryContent)> {
        let draft = self
            .draft_repository
            .get(id)
            .ok_or(ServiceError::DraftNotFound)?;
        let content = self
            .draft_content_repository
            .get(id)
            .ok_or(ServiceError::DraftNotFound)?;
        Ok((draft, content))
    }

    pub fn get_drafts(&self, identity: Principal) -> ServiceResult<Vec<Draft>> {
        self.draft_repository
            .get_drafts_by_author(identity)
            .map_err(|e| ServiceError::InternalError {
                reason: format!("Failed to get drafts: {}", e),
            })
    }

    // debug only
    pub fn debug_drafts(&self) -> (Vec<Draft>, Vec<StoryContent>) {
        let drafts = self.draft_repository.get_all();
        let contents = self.draft_content_repository.get_all();
        (drafts, contents)
    }
}

fn validate_draft_author(author: Principal, identity: Principal) -> Result<(), ServiceError> {
    if author != identity {
        return Err(ServiceError::IdentityUnauthorized {
            identity: identity.to_string(),
        });
    }
    Ok(())
}

fn validate_empty_save_args(args: &SaveDraftArgs, message: &str) -> Result<(), ServiceError> {
    if args.title.is_none() && args.content.is_none() && args.detail.is_none() {
        return Err(ServiceError::UnprocessableEntity {
            reason: message.to_string(),
        });
    }
    Ok(())
}
