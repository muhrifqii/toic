use std::sync::Arc;

use candid::{Nat, Principal};
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use itertools::Itertools;
use lazy_static::lazy_static;

#[cfg(all(test, not(rust_analyzer)))]
use crate::utils::mocks::{caller, timestamp};
#[cfg(any(not(test), rust_analyzer))]
use crate::utils::timestamp;
use crate::{
    repositories::story::{
        StoryContentRepository, StoryRepository, STORY_CONTENT_REPOSITORY, STORY_REPOSITORY,
    },
    services::user::USER_SERVICE,
    structure::{AuditableRepository, BinaryTreeRepository},
    token::{LedgerService, LEDGER_SERVICE},
    types::{
        Category, RepositoryError, Score, ServiceError, ServiceResult, SortOrder, Story,
        StoryContent, StoryInteractionArgs, SupportSize,
    },
};

use super::{
    llm::{expand_paragraph, write_story_description},
    user::{self, UserService},
};

pub const MAX_STORY_SUPPORT_GIVEN: SupportSize = 10;

lazy_static! {
    pub static ref STORY_SERVICE: Arc<StoryService> = Arc::new(StoryService::new(
        STORY_REPOSITORY.clone(),
        STORY_CONTENT_REPOSITORY.clone(),
        LEDGER_SERVICE.clone(),
        USER_SERVICE.clone(),
    ));
}

#[derive(Debug)]
pub struct StoryService {
    story_repository: Arc<StoryRepository>,
    story_content_repository: Arc<StoryContentRepository>,
    ledger_service: Arc<LedgerService>,
    user_service: Arc<UserService>,
}

impl StoryService {
    pub fn new(
        story_repository: Arc<StoryRepository>,
        story_content_repository: Arc<StoryContentRepository>,
        ledger_service: Arc<LedgerService>,
        user_service: Arc<UserService>,
    ) -> Self {
        Self {
            story_repository,
            story_content_repository,
            ledger_service,
            user_service,
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

    pub async fn support_story(
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
        let user_supporter = self.user_service.get_user(&identity)?;
        let category_scoring = calculate_category_matching_score(
            &story.detail.category,
            &user_supporter.followed_categories,
        );
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
        story.score = calculate_basic_scoring(support_given, tip_given.clone(), category_scoring);

        self.story_repository
            .update(story.clone())
            .and_then(|_| {
                self.story_repository
                    .support_story(args.id, identity, support_given, tip_given)
            })
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

    pub fn get_recommended_stories(
        &self,
        cursor: Option<(Score, u64)>,
        limit: usize,
    ) -> ServiceResult<(Option<(Score, u64)>, Vec<Story>)> {
        let now = timestamp();
        let stories = self
            .story_repository
            .get_stories_by_score(cursor, limit)
            .map_err(map_story_err)?;
        let next_cursor = stories.last().map(|s| (s.score, s.id));
        let stories = stories
            .into_iter()
            .sorted_by_cached_key(|s| calculate_complete_scoring(s.score, s.created_at, now))
            .rev()
            .collect_vec();
        Ok((next_cursor, stories))
    }

    pub fn get_story_supporter(&self, id: u64) -> ServiceResult<Vec<Principal>> {
        let supporters = self
            .story_repository
            .get_story_supporters(id)
            .map_err(map_story_err)?;
        let supporters = supporters.iter().map(|(s, _, _)| *s).collect();
        Ok(supporters)
    }

    pub async fn assist_expand_writing(
        &self,
        id: &u64,
        identity: &Principal,
    ) -> ServiceResult<String> {
        self.user_service.ensure_ai_enabled(&identity)?;
        let content = self
            .story_content_repository
            .get(id)
            .ok_or(ServiceError::StoryNotFound)?;
        let expansion = expand_paragraph(content.content)
            .await
            .map_err(|e| ServiceError::AiModelError(e))?;
        Ok(expansion)
    }

    pub async fn assist_story_description(
        &self,
        id: &u64,
        identity: &Principal,
    ) -> ServiceResult<String> {
        self.user_service.ensure_ai_enabled(&identity)?;
        let content = self
            .story_content_repository
            .get(id)
            .ok_or(ServiceError::StoryNotFound)?;
        let description = write_story_description(content.content)
            .await
            .map_err(|e| ServiceError::AiModelError(e))?;
        Ok(description)
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

fn calculate_time_bonus_scoring(created_at: u64, now: u64) -> u64 {
    let ages_seconds = (now - created_at) / 1_000_000_000;
    let time_bonus = (3660 - (ages_seconds / (3 * 60 * 60 * 24)) as i64 * 10).max(0) as u64;
    time_bonus
}

fn calculate_basic_scoring(support: u32, tips: Nat, category_matching_score: u64) -> u64 {
    let support_score = support as u64 * 10;
    let tip_score = (tips.0 * 100_usize).try_into().unwrap_or(u64::MAX);

    support_score
        .saturating_add(tip_score)
        .saturating_add(category_matching_score)
}

fn calculate_complete_scoring(basic_scoring: u64, created_at: u64, now: u64) -> u64 {
    basic_scoring.saturating_add(calculate_time_bonus_scoring(created_at, now))
}

fn calculate_category_matching_score(category: &Category, categories: &[Category]) -> u64 {
    if categories.contains(category) {
        100
    } else {
        0
    }
}
