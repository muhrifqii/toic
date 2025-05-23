use std::sync::Arc;

use candid::Principal;
use lazy_static::lazy_static;

use crate::{
    repositories::user::{UserRepository, USER_REPOSITORY},
    structure::{BinaryTreeRepository, Repository},
    token::{LedgerService, LEDGER_SERVICE},
    types::{Category, OnboardingArgs, RepositoryError, ServiceError, ServiceResult, User},
};

lazy_static! {
    pub static ref USER_SERVICE: Arc<UserService> = Arc::new(UserService::new(
        USER_REPOSITORY.clone(),
        LEDGER_SERVICE.clone()
    ));
}

#[derive(Debug)]
pub struct UserService {
    user_repository: Arc<UserRepository>,
    ledger_service: Arc<LedgerService>,
}

impl UserService {
    pub fn new(user_repository: Arc<UserRepository>, ledger_service: Arc<LedgerService>) -> Self {
        Self {
            user_repository,
            ledger_service,
        }
    }

    pub fn register(&self, identity: Principal, created_at: u64) -> ServiceResult<User> {
        if self.user_repository.exists(&identity) {
            return Err(ServiceError::Conflict {
                entity: "User already exists.".to_string(),
            });
        }
        let user = User::new(identity, created_at);
        let user = self.user_repository.insert(user).map_err(map_user_err)?;
        Ok(user)
    }

    pub fn get_user(&self, identity: &Principal) -> ServiceResult<User> {
        self.user_repository
            .get(identity)
            .ok_or(ServiceError::IdentityNotFound {
                identity: identity.to_string(),
            })
    }

    pub fn complete_onboarding(
        &self,
        identity: Principal,
        args: OnboardingArgs,
    ) -> ServiceResult<()> {
        if args.categories.len() != 3 {
            return Err(ServiceError::UnprocessableEntity {
                reason: "You must select exactly 3 categories.".to_string(),
            });
        }

        let mut user =
            self.user_repository
                .get(&identity)
                .ok_or(ServiceError::IdentityNotFound {
                    identity: identity.to_string(),
                })?;
        if user.onboarded {
            return Err(ServiceError::UnprocessableEntity {
                reason: "You have already completed onboarding.".to_string(),
            });
        }
        user.name = args.name;
        user.bio = args.bio;
        user.followed_categories = args.categories;
        user.onboarded = true;
        self.user_repository.update(user).map_err(map_user_err)?;
        Ok(())
    }

    pub fn ensure_ai_enabled(&self, identity: &Principal) -> ServiceResult<()> {
        let staked = self
            .ledger_service
            .locked_balance_of(identity.clone().into());
        if staked < 1_000_000_usize {
            return Err(ServiceError::UnprocessableEntity {
                reason: "You need to stake at least 1 million of your token to unlock this feature"
                    .to_string(),
            });
        }
        Ok(())
    }
}

fn map_user_err(e: RepositoryError) -> ServiceError {
    match e {
        RepositoryError::NotFound => ServiceError::IdentityNotFound {
            identity: "User not found.".to_string(),
        },
        RepositoryError::Conflict => ServiceError::Conflict {
            entity: "User".to_string(),
        },
        RepositoryError::IllegalArgument { reason } => ServiceError::UnprocessableEntity {
            reason: reason.to_string(),
        },
        _ => ServiceError::InternalError {
            reason: format!("{:?}", e),
        },
    }
}
