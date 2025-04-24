use std::sync::Arc;

use candid::Principal;

use crate::{
    repositories::user::UserRepository,
    structure::{BinaryTreeRepository, Repository},
    types::{Category, ServiceError, ServiceResult, User},
};

pub struct AccountService {
    account_repository: Arc<UserRepository>,
}

impl AccountService {
    pub fn new(account_repository: Arc<UserRepository>) -> Self {
        Self { account_repository }
    }

    // pub fn register_account(&self, identity: Principal, timestamp: u64) -> ServiceResult<Account> {
    //     if self.account_repository.exists(&identity) {
    //         return Err(ServiceError::Conflict {
    //             entity: "".to_string(),
    //         });
    //     }

    //     let account = Account::new(identity, timestamp);
    //     self.account_repository
    //         .insert(account)
    //         .map_err(|e| ServiceError::Conflict { reason: e })
    // }

    pub fn get_account(&self, identity: Principal) -> ServiceResult<User> {
        self.account_repository
            .get(&identity)
            .ok_or(ServiceError::IdentityNotFound {
                identity: identity.to_string(),
            })
    }

    pub fn complete_onboarding(
        &self,
        identity: Principal,
        selected_categories: Vec<Category>,
    ) -> ServiceResult<()> {
        if selected_categories.len() != 3 {
            return Err(ServiceError::UnprocessableEntity {
                reason: "You must select exactly 3 categories.".to_string(),
            });
        }

        let mut user =
            self.account_repository
                .get(&identity)
                .ok_or(ServiceError::IdentityNotFound {
                    identity: identity.to_string(),
                })?;
        if user.onboarded {
            return Err(ServiceError::UnprocessableEntity {
                reason: "You have already completed onboarding.".to_string(),
            });
        }
        user.followed_categories = selected_categories;
        user.onboarded = true;
        self.account_repository
            .update(user)
            .map_err(|e| ServiceError::InternalError {
                reason: e.to_string(),
            })?;
        Ok(())
    }
}
