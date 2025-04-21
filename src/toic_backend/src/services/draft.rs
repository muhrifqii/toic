use std::sync::Arc;

use candid::Principal;
use lazy_static::lazy_static;

use crate::{
    repositories::{
        draft::{DraftRepository, DRAFT_REPOSITORY},
        story::{StoryRepository, STORY_REPOSITORY},
    },
    structure::{AuditableRepository, BinaryTreeRepository},
    types::{
        Draft, RepositoryError, SaveDraftArgs, ServiceError, ServiceResult, Story, StoryDetail,
    },
    utils::estimate_read_time,
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

    pub async fn create_draft(
        &self,
        args: SaveDraftArgs,
        identity: Principal,
    ) -> ServiceResult<Draft> {
        validate_empty_save_args(&args, "Nothing to save")?;

        let mut draft = Draft::new(
            args.title.unwrap_or_default(),
            args.detail,
            args.content.unwrap_or_default(),
            identity,
        );
        set_estimated_read_time(&mut draft);

        self.draft_repository.insert(draft).map_err(|e| match e {
            RepositoryError::Conflict => ServiceError::Conflict {
                entity: "Draft".to_string(),
            },
            _ => ServiceError::InternalError {
                reason: format!("Failed to create draft: {}", e),
            },
        })
    }

    pub async fn update_draft(
        &self,
        id: u64,
        args: SaveDraftArgs,
        identity: Principal,
    ) -> ServiceResult<Draft> {
        validate_empty_save_args(&args, "Nothing to update")?;

        let mut draft = self
            .draft_repository
            .get(&id)
            .ok_or(ServiceError::DraftNotFound)?;
        validate_draft_author(&draft, identity)?;

        if let Some(new_title) = args.title {
            draft.title = new_title;
        }
        if let Some(new_content) = args.content {
            draft.content = new_content;
        }
        if args.detail.is_some() {
            draft.detail = args.detail;
        }
        set_estimated_read_time(&mut draft);

        self.draft_repository.update(draft).map_err(|e| match e {
            RepositoryError::NotFound => ServiceError::DraftNotFound,
            _ => ServiceError::InternalError {
                reason: format!("Failed to update draft: {}", e),
            },
        })
    }

    pub async fn publish_draft(&self, id: u64, identity: Principal) -> ServiceResult<Story> {
        let draft = self
            .draft_repository
            .get(&id)
            .ok_or(ServiceError::DraftNotFound)?;
        validate_draft_author(&draft, identity)?;
        if draft.title.is_empty() || draft.content.is_empty() {
            return Err(ServiceError::UnprocessableEntity {
                reason: "Title and content cannot be empty".to_string(),
            });
        }
        let detail = draft
            .detail
            .clone()
            .ok_or_else(|| ServiceError::UnprocessableEntity {
                reason: "Story detail is required".to_string(),
            })?;

        // Promote to story
        let story = Story::new(draft, detail);

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

    pub async fn delete_draft(&self, id: u64, identity: Principal) -> ServiceResult<u64> {
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

fn set_estimated_read_time(draft: &mut Draft) {
    let content_len = draft.content.len();
    if content_len > 200 {
        draft.read_time = estimate_read_time(&draft.content);
    } else if content_len > 0 {
        draft.read_time = 1;
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

fn validate_empty_save_args(args: &SaveDraftArgs, message: &str) -> Result<(), ServiceError> {
    if args.title.is_none() && args.content.is_none() && args.detail.is_none() {
        return Err(ServiceError::UnprocessableEntity {
            reason: message.to_string(),
        });
    }
    Ok(())
}
