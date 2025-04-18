use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::{
    memory_manager::{MemoryId, VirtualMemory},
    storable::Bound,
    BTreeMap, Cell, DefaultMemoryImpl, Storable,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type VMemory = VirtualMemory<DefaultMemoryImpl>;
pub type SerialRefCell = RefCell<Cell<u64, VMemory>>;
pub type BTreeMapRefCell<K, V> = RefCell<BTreeMap<K, V, VMemory>>;

pub const SERIAL_STORY_MEM_ID: MemoryId = MemoryId::new(1);
pub const SERIAL_DRAFT_MEM_ID: MemoryId = MemoryId::new(2);
pub const ET_STORY_MEM_ID: MemoryId = MemoryId::new(3);
pub const ET_DRAFT_MEM_ID: MemoryId = MemoryId::new(4);
pub const IDX_DRAFT_AUTHOR_MEM_ID: MemoryId = MemoryId::new(5);

pub type RepositoryResult<T> = Result<T, RepositoryError>;
pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Error, Debug, Eq, PartialEq, Clone)]
pub enum RepositoryError {
    #[error(r#"The requested entity was not found in the repository."#)]
    NotFound,
    #[error(r#"Cannot write on existing entity."#)]
    Conflict,
    #[error(r#"Invalid update operation: {reason}."#)]
    IllegalUpdate { reason: String },
}

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum ServiceError {
    #[error(r#"User identity {identity} cannot be found."#)]
    IdentityNotFound { identity: String },
    #[error(r#"User identity {identity} is not authorized to perform this action."#)]
    IdentityUnauthorized { identity: String },
    #[error("Internal error: {reason}")]
    InternalError { reason: String },
    #[error("Draft not found")]
    DraftNotFound,
    #[error("Unprocessable entity: {reason}")]
    UnprocessableEntity { reason: String },
}

pub trait AuditableEntity {
    fn id(&self) -> u64;
    fn set_created_at(&mut self, created_at: u64);
    fn set_updated_at(&mut self, updated_at: u64);
}

#[derive(Debug, CandidType, Deserialize, Serialize, Clone)]
pub enum StoryLabel {
    /// Original Content
    OC,
    /// Collaborated with AI
    AI,
}

#[derive(Debug, CandidType, Deserialize, Serialize, Clone)]
pub struct Story {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub label: StoryLabel,
    pub author: Principal,
    pub total_score: u32,
    pub created_at: u64,
}

impl Storable for Story {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut encoded = Vec::new();
        ciborium::into_writer(self, &mut encoded).unwrap();
        std::borrow::Cow::Owned(encoded)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        ciborium::from_reader(bytes.as_ref()).unwrap()
    }
    const BOUND: Bound = Bound::Unbounded;
}

impl AuditableEntity for Story {
    fn id(&self) -> u64 {
        self.id
    }

    fn set_created_at(&mut self, created_at: u64) {
        self.created_at = created_at;
    }

    fn set_updated_at(&mut self, _updated_at: u64) {}
}

impl Story {
    pub fn new(title: String, content: String, label: StoryLabel, author: Principal) -> Self {
        Self {
            id: 0,
            title,
            content,
            label,
            author,
            total_score: 0,
            created_at: 0,
        }
    }
}

#[derive(Debug, CandidType, Deserialize, Serialize, Clone)]
pub struct Draft {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub author: Principal,
    pub created_at: u64,
    pub updated_at: u64,
    pub ai_used: bool,
}

impl Storable for Draft {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut encoded = Vec::new();
        ciborium::into_writer(self, &mut encoded).unwrap();
        std::borrow::Cow::Owned(encoded)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        ciborium::from_reader(bytes.as_ref()).unwrap()
    }
    const BOUND: Bound = Bound::Unbounded;
}

impl AuditableEntity for Draft {
    fn id(&self) -> u64 {
        self.id
    }

    fn set_created_at(&mut self, created_at: u64) {
        self.created_at = created_at;
    }

    fn set_updated_at(&mut self, updated_at: u64) {
        self.updated_at = updated_at;
    }
}

impl Draft {
    pub fn new(title: String, content: String, author: Principal, ai_used: bool) -> Self {
        Self {
            id: 0,
            title,
            content,
            author,
            created_at: 0,
            updated_at: 0,
            ai_used,
        }
    }
}
