use std::{cell::RefCell, sync::Arc};

use candid::Principal;
use ic_stable_structures::BTreeMap;
use lazy_static::lazy_static;

use crate::{
    memory::{ET_USER_MEM_ID, MEMORY_MANAGER},
    structure::{BinaryTreeRepository, Repository},
    types::{BTreeMapRefCell, RepositoryError, RepositoryResult, User, VMemory},
    utils::timestamp,
};

thread_local! {
    static USER: BTreeMapRefCell<Principal, User> = RefCell::new(
        BTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(ET_USER_MEM_ID))
        )
    );
}

lazy_static! {
    pub static ref USER_REPOSITORY: Arc<UserRepository> = Arc::new(UserRepository::default());
}

#[derive(Debug, Default)]
pub struct UserRepository;

impl BinaryTreeRepository<Principal, User, VMemory> for UserRepository {
    fn with_ref<F, R>(f: F) -> R
    where
        F: FnOnce(&RefCell<BTreeMap<Principal, User, VMemory>>) -> R,
    {
        USER.with(f)
    }
}

impl Repository<Principal, User, VMemory> for UserRepository {
    fn insert(&self, mut value: User) -> RepositoryResult<User> {
        if Self::with_ref(|cell| cell.borrow().contains_key(&value.id)) {
            return Err(RepositoryError::Conflict);
        }

        let now = timestamp();
        value.created_at = now;
        Self::with_ref(|cell| cell.borrow_mut().insert(value.id, value.clone()));
        Ok(value)
    }

    fn update(&self, value: User) -> RepositoryResult<User> {
        if Self::with_ref(|cell| !cell.borrow().contains_key(&value.id)) {
            return Err(RepositoryError::NotFound);
        }

        Self::with_ref(|cell| cell.borrow_mut().insert(value.id, value.clone()));
        Ok(value)
    }
}
