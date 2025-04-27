use candid::Principal;
use ic_cdk::{caller, export_candid, query, update};
use services::{draft::DRAFT_SERVICE, story::STORY_SERVICE, user::USER_SERVICE};

mod memory;
mod repositories;
mod services;
mod structure;
mod token;
mod types;
mod utils;

use token::*;
use types::*;
use utils::timestamp;

fn get_and_validate_caller() -> ServiceResult<Principal> {
    let identity = caller();
    if identity == Principal::anonymous() {
        return Err(ServiceError::IdentityUnauthorized {
            identity: identity.to_string(),
        });
    }
    Ok(identity)
}

#[update]
async fn create_draft(args: SaveDraftArgs) -> ServiceResult<Draft> {
    let identity = get_and_validate_caller()?;

    DRAFT_SERVICE.create_draft(args, identity).await
}

#[update]
async fn update_draft(id: u64, args: SaveDraftArgs) -> ServiceResult<()> {
    let identity = get_and_validate_caller()?;

    DRAFT_SERVICE.update_draft(id, args, identity).await
}

#[update]
async fn publish_draft(id: u64) -> ServiceResult<Story> {
    let identity = get_and_validate_caller()?;

    DRAFT_SERVICE.publish_draft(id, identity).await
}

#[update]
async fn delete_draft(id: u64) -> ServiceResult<()> {
    let identity = get_and_validate_caller()?;

    DRAFT_SERVICE.delete_draft(id, identity).await.map(|_| ())
}

#[query]
fn get_draft(id: u64) -> ServiceResult<(Draft, StoryContent)> {
    get_and_validate_caller()?;

    DRAFT_SERVICE.get_draft(&id)
}

#[query]
fn get_drafts() -> ServiceResult<Vec<Draft>> {
    let identity = get_and_validate_caller()?;

    DRAFT_SERVICE.get_drafts(identity)
}

#[query]
fn get_story(id: u64) -> ServiceResult<(Story, StoryContent)> {
    // anon can read

    STORY_SERVICE.get_story(&id)
}

#[query]
fn get_recommended_stories(
    args: FetchStoriesByScoreArgs,
) -> ServiceResult<(Option<(Score, u64)>, Vec<Story>)> {
    get_and_validate_caller()?;

    STORY_SERVICE.get_recommended_stories(args.cursor, args.limit.unwrap_or(15))
}

#[query]
fn get_stories_by_author(args: FetchStoriesArgs) -> ServiceResult<(Option<u64>, Vec<Story>)> {
    // anon can read

    if args.author.is_none() {
        return Err(ServiceError::UnprocessableEntity {
            reason: "Author is required.".to_string(),
        });
    }

    STORY_SERVICE.get_stories_by_author(args.author.unwrap(), args.cursor, args.limit.unwrap_or(15))
}

#[query]
fn get_stories_by_category(args: FetchStoriesArgs) -> ServiceResult<(Option<u64>, Vec<Story>)> {
    // anon can read

    if args.category.is_none() {
        return Err(ServiceError::UnprocessableEntity {
            reason: "Category is required.".to_string(),
        });
    }

    STORY_SERVICE.get_stories_by_category(
        args.category.unwrap(),
        args.cursor,
        args.limit.unwrap_or(15),
    )
}

#[update]
async fn support_story(args: StoryInteractionArgs) -> ServiceResult<()> {
    let identity = get_and_validate_caller()?;

    STORY_SERVICE.support_story(args, identity).await
}

#[query]
fn get_story_supporter(id: u64) -> ServiceResult<Vec<UserOutline>> {
    get_and_validate_caller()?;

    STORY_SERVICE.get_story_supporter(id)
}

#[update]
async fn assist_action(args: AssistActionArgs) -> ServiceResult<String> {
    let identity = &get_and_validate_caller()?;

    match args {
        AssistActionArgs::ExpandWriting(id) => {
            STORY_SERVICE.assist_expand_writing(&id, identity).await
        }
        AssistActionArgs::GenerateDescription(id) => {
            STORY_SERVICE.assist_story_description(&id, identity).await
        }
    }
}

#[update]
async fn login() -> ServiceResult<User> {
    let identity = get_and_validate_caller()?;

    match USER_SERVICE.get_user(&identity) {
        Ok(user) => Ok(user),
        Err(ServiceError::IdentityNotFound { .. }) => USER_SERVICE.register(identity, timestamp()),
        Err(e) => Err(e),
    }
}

#[update]
async fn complete_onboarding(args: OnboardingArgs) -> ServiceResult<()> {
    let identity = get_and_validate_caller()?;

    USER_SERVICE.complete_onboarding(identity, args)?;

    LEDGER_SERVICE
        .mint(TransferArg {
            from_subaccount: None,
            to: identity.into(),
            fee: None,
            created_at_time: None,
            memo: None,
            amount: 1000_usize.into(),
        })
        .map_err(|e| ServiceError::InternalError {
            reason: format!("{:?}", e),
        })?;

    Ok(())
}

export_candid!();
