use std::sync::Arc;

use candid::Principal;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use lazy_static::lazy_static;

#[cfg(all(test, not(rust_analyzer)))]
use crate::utils::mocks::{caller, timestamp};
#[cfg(any(not(test), rust_analyzer))]
use crate::utils::timestamp;
use crate::{
    repositories::story::{
        StoryContentRepository, StoryRepository, STORY_CONTENT_REPOSITORY, STORY_REPOSITORY,
    },
    structure::{AuditableRepository, BinaryTreeRepository},
    token::{LedgerService, LEDGER_SERVICE},
    types::{
        Category, RepositoryError, ServiceError, ServiceResult, SortOrder, Story, StoryContent,
        StoryInteractionArgs, SupportSize,
    },
};

pub const MAX_STORY_SUPPORT_GIVEN: SupportSize = 100;

lazy_static! {
    pub static ref STORY_SERVICE: Arc<StoryService> = Arc::new(StoryService::new(
        STORY_REPOSITORY.clone(),
        STORY_CONTENT_REPOSITORY.clone(),
        LEDGER_SERVICE.clone(),
    ));
}

#[derive(Debug, Default)]
pub struct StoryService {
    story_repository: Arc<StoryRepository>,
    story_content_repository: Arc<StoryContentRepository>,
    ledger_service: Arc<LedgerService>,
}

impl StoryService {
    pub fn new(
        story_repository: Arc<StoryRepository>,
        story_content_repository: Arc<StoryContentRepository>,
        ledger_service: Arc<LedgerService>,
    ) -> Self {
        Self {
            story_repository,
            story_content_repository,
            ledger_service,
        }
    }

    pub fn get_story(&self, id: &u64) -> ServiceResult<(Story, StoryContent)> {
        let story = self
            .story_repository
            .get(id)
            .ok_or(ServiceError::StoryNotFound)?;
        let content = self
            .story_content_repository
            .get(id)
            .ok_or(ServiceError::StoryNotFound)?;
        Ok((story, content))
    }

    pub fn support_story(
        &self,
        args: StoryInteractionArgs,
        identity: Principal,
    ) -> ServiceResult<()> {
        if args.support.is_none() && args.tip.is_none() {
            // non-strict check, fail early with success result
            return Ok(());
        }

        let mut story = self
            .story_repository
            .get(&args.id)
            .ok_or(ServiceError::StoryNotFound)?;
        validate_supporter(story.author, identity)?;
        let (mut support_given, mut tip_given) = self
            .story_repository
            .get_story_supporter_size(args.id, identity)
            .map_err(map_story_err)?
            .unwrap_or_default();
        if support_given >= MAX_STORY_SUPPORT_GIVEN {
            return Ok(());
        }
        if let Some(mut new_support) = args.support {
            if new_support + support_given > MAX_STORY_SUPPORT_GIVEN {
                new_support = MAX_STORY_SUPPORT_GIVEN - support_given;
            }
            story.total_support += new_support;
            support_given += new_support;
        }
        if let Some(new_tip) = args.tip {
            self.ledger_service
                .transfer(TransferArg {
                    from_subaccount: None,
                    to: story.author.into(),
                    fee: None,
                    created_at_time: Some(timestamp()),
                    memo: None,
                    amount: new_tip.clone(),
                })
                .map_err(map_transfer_err)?;
            story.total_tip_support += new_tip.clone();
            tip_given += new_tip;
            // tip given is calculated before fee
        }

        self.story_repository
            .update(story.clone())
            .map_err(map_story_err)?;
        self.story_repository
            .support_story(args.id, identity, support_given, tip_given)
            .map_err(map_story_err)?;
        Ok(())
    }

    pub fn get_stories_by_author(
        &self,
        author: Principal,
        cursor: Option<u64>,
        limit: usize,
    ) -> ServiceResult<(Option<u64>, Vec<Story>)> {
        let stories = self
            .story_repository
            .get_stories_by_author(author, cursor, limit)
            .map_err(map_story_err)?;
        Ok((stories.last().map(|s| s.id), stories))
    }

    pub fn get_stories_by_category(
        &self,
        category: Category,
        cursor: Option<u64>,
        limit: usize,
    ) -> ServiceResult<(Option<u64>, Vec<Story>)> {
        let stories = self
            .story_repository
            .get_stories_by_categories(vec![category], SortOrder::default(), vec![cursor], limit)
            .map_err(map_story_err)?;
        Ok((stories.last().map(|s| s.id), stories))
    }

    pub fn get_story_supporter(&self, id: u64) -> ServiceResult<Vec<Principal>> {
        let supporters = self
            .story_repository
            .get_story_supporters(id)
            .map_err(map_story_err)?;
        let supporters = supporters.iter().map(|(s, _, _)| *s).collect();
        Ok(supporters)
    }
}

fn validate_supporter(author: Principal, supporter: Principal) -> ServiceResult<()> {
    if author == supporter {
        return Err(ServiceError::UnprocessableEntity {
            reason: "You cannot support your own story.".to_string(),
        });
    }
    Ok(())
}

fn map_story_err(e: RepositoryError) -> ServiceError {
    match e {
        RepositoryError::NotFound => ServiceError::StoryNotFound,
        RepositoryError::IllegalArgument { reason } => ServiceError::UnprocessableEntity {
            reason: reason.to_string(),
        },
        _ => ServiceError::InternalError {
            reason: format!("{:?}", e),
        },
    }
}

fn map_transfer_err(e: TransferError) -> ServiceError {
    ServiceError::TransferError {
        reason: format!("{:?}", e),
    }
}
