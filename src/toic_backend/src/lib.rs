use candid::Principal;
use ic_cdk::{caller, export_candid, query, update};
use services::draft::DRAFT_SERVICE;

mod memory;
mod repositories;
mod services;
mod structure;
mod token;
mod types;
mod utils;

use token::*;
use types::*;

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
async fn update_draft(id: u64, args: SaveDraftArgs) -> ServiceResult<Draft> {
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
fn get_draft(id: u64) -> ServiceResult<Draft> {
    let identity = get_and_validate_caller()?;

    DRAFT_SERVICE.get_draft(&id)
}

#[query]
fn get_drafts(cursor: Option<u64>, limit: usize) -> ServiceResult<(Option<u64>, Vec<Draft>)> {
    let identity = get_and_validate_caller()?;

    DRAFT_SERVICE.get_drafts(identity, cursor, limit)
}

export_candid!();
