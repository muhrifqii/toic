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

fn get_and_validate_caller() -> ApiResult<Principal> {
    let identity = caller();
    if identity == Principal::anonymous() {
        return Err(ServiceError::IdentityUnauthorized {
            identity: identity.to_string(),
        })
        .map_err(api_err);
    }
    Ok(identity)
}

fn api_err(err: ServiceError) -> ErrorResponse {
    return ErrorResponse {
        message: err.to_string(),
    };
}

#[update]
async fn create_draft(args: SaveDraftArgs) -> ApiResult<Draft> {
    let identity = get_and_validate_caller()?;

    DRAFT_SERVICE
        .create_draft(args, identity)
        .await
        .map_err(api_err)
}

#[update]
async fn update_draft(id: u64, args: SaveDraftArgs) -> ApiResult<()> {
    let identity = get_and_validate_caller()?;

    DRAFT_SERVICE
        .update_draft(id, args, identity)
        .await
        .map_err(api_err)
}

#[update]
async fn publish_draft(id: u64) -> ApiResult<Story> {
    let identity = get_and_validate_caller()?;

    DRAFT_SERVICE
        .publish_draft(id, identity)
        .await
        .map_err(api_err)
}

#[update]
async fn delete_draft(id: u64) -> ApiResult<()> {
    let identity = get_and_validate_caller()?;

    DRAFT_SERVICE
        .delete_draft(id, identity)
        .await
        .map_err(api_err)
        .map(|_| ())
}

#[query]
fn get_draft(id: u64) -> ApiResult<(Draft, StoryContent)> {
    get_and_validate_caller()?;

    DRAFT_SERVICE.get_draft(&id).map_err(api_err)
}

#[query]
fn get_drafts() -> ApiResult<Vec<Draft>> {
    let identity = get_and_validate_caller()?;

    DRAFT_SERVICE.get_drafts(identity).map_err(api_err)
}

#[query]
fn get_story(id: u64) -> ApiResult<(Story, StoryContent)> {
    // anon can read

    STORY_SERVICE.get_story(&id).map_err(api_err)
}

#[query]
fn get_recommended_stories(
    args: FetchStoriesByScoreArgs,
) -> ApiResult<(Option<(Score, u64)>, Vec<Story>)> {
    get_and_validate_caller()?;

    STORY_SERVICE
        .get_recommended_stories(args.cursor, args.limit.unwrap_or(15))
        .map_err(api_err)
}

#[query]
fn get_stories_by_author(args: FetchStoriesArgs) -> ApiResult<(Option<u64>, Vec<Story>)> {
    // anon can read

    if args.author.is_none() {
        return Err(ServiceError::UnprocessableEntity {
            reason: "Author is required.".to_string(),
        })
        .map_err(api_err);
    }

    STORY_SERVICE
        .get_stories_by_author(args.author.unwrap(), args.cursor, args.limit.unwrap_or(15))
        .map_err(api_err)
}

#[query]
fn get_stories_by_category(args: FetchStoriesArgs) -> ApiResult<(Option<u64>, Vec<Story>)> {
    // anon can read

    if args.category.is_none() {
        return Err(ServiceError::UnprocessableEntity {
            reason: "Category is required.".to_string(),
        })
        .map_err(api_err);
    }

    STORY_SERVICE
        .get_stories_by_category(
            args.category.unwrap(),
            args.cursor,
            args.limit.unwrap_or(15),
        )
        .map_err(api_err)
}

#[update]
async fn support_story(args: StoryInteractionArgs) -> ApiResult<()> {
    let identity = get_and_validate_caller()?;

    STORY_SERVICE
        .support_story(args, identity)
        .await
        .map_err(api_err)
}

#[query]
fn get_story_supporter(id: u64) -> ApiResult<Vec<UserOutline>> {
    get_and_validate_caller()?;

    STORY_SERVICE.get_story_supporter(id).map_err(api_err)
}

#[update]
async fn assist_action(args: AssistActionArgs) -> ApiResult<String> {
    let identity = &get_and_validate_caller()?;

    match args {
        AssistActionArgs::ExpandWriting(id) => STORY_SERVICE
            .assist_expand_writing(&id, identity)
            .await
            .map_err(api_err),
        AssistActionArgs::GenerateDescription(id) => STORY_SERVICE
            .assist_story_description(&id, identity)
            .await
            .map_err(api_err),
    }
}

#[update]
async fn login() -> ApiResult<User> {
    let identity = get_and_validate_caller()?;

    match USER_SERVICE.get_user(&identity) {
        Ok(user) => Ok(user),
        Err(ServiceError::IdentityNotFound { .. }) => USER_SERVICE
            .register(identity, timestamp())
            .map_err(api_err),
        Err(e) => Err(e).map_err(api_err),
    }
}

#[update]
async fn complete_onboarding(args: OnboardingArgs) -> ApiResult<()> {
    let identity = get_and_validate_caller()?;

    USER_SERVICE
        .complete_onboarding(identity, args)
        .map_err(api_err)?;

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
        })
        .map_err(api_err)?;

    Ok(())
}

export_candid!();
