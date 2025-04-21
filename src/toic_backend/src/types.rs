use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::{storable::Bound, BTreeMap, Cell, Storable};
use serde::{Deserialize, Serialize};
use strum::EnumString;
use thiserror::Error;

pub use crate::memory::VMemory;

pub type SerialRefCell = RefCell<Cell<u64, VMemory>>;
pub type BTreeMapRefCell<K, V> = RefCell<BTreeMap<K, V, VMemory>>;

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

#[derive(Error, Debug, PartialEq, Eq, Clone, CandidType)]
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
    #[error("{entity} already exists")]
    Conflict { entity: String },
}

pub trait AuditableEntity {
    fn id(&self) -> u64;
    fn set_created_at(&mut self, created_at: u64);
    fn set_updated_at(&mut self, updated_at: u64);
}

#[derive(
    Debug, Clone, CandidType, Deserialize, Serialize, EnumString, PartialEq, PartialOrd, Eq, Ord,
)]
pub enum Category {
    SciFi,
    Fantasy,
    Comedy,
    Romance,
    Horror,
    Thriller,
    Crime,
    Adventure,
    NonFiction,
    Biography,
}

impl Storable for Category {
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

#[derive(Debug, CandidType, Deserialize, Serialize, Clone)]
pub struct StoryDetail {
    pub description: String,
    pub mature_content: bool,
    pub category: Category,
}

impl StoryDetail {
    pub fn new(description: String, mature_content: bool, category: Category) -> Self {
        Self {
            description,
            mature_content,
            category,
        }
    }
}

#[derive(Debug, CandidType, Deserialize, Serialize, Clone)]
pub struct Story {
    pub id: u64,
    pub title: String,
    pub detail: StoryDetail,
    pub content: String,
    pub author: Principal,
    pub total_support: u32,
    pub total_views: u32,
    pub created_at: u64,
    pub updated_at: Option<u64>,
    pub read_time: u32,
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

    fn set_updated_at(&mut self, updated_at: u64) {
        self.updated_at = Some(updated_at);
    }
}

impl Story {
    pub fn new(draft: Draft, detail: StoryDetail) -> Self {
        Self {
            id: 0,
            title: draft.title,
            detail,
            content: draft.content,
            author: draft.author,
            total_support: 0,
            total_views: 0,
            created_at: 0,
            updated_at: None,
            read_time: draft.read_time,
        }
    }
}

#[derive(Debug, CandidType, Deserialize, Serialize, Clone)]
pub struct Draft {
    pub id: u64,
    pub title: String,
    pub detail: Option<StoryDetail>,
    pub content: String,
    pub author: Principal,
    pub created_at: u64,
    pub updated_at: Option<u64>,
    pub read_time: u32,
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
        self.updated_at = Some(updated_at);
    }
}

impl Draft {
    pub fn new(
        title: String,
        detail: Option<StoryDetail>,
        content: String,
        author: Principal,
    ) -> Self {
        Self {
            id: 0,
            title,
            detail,
            content,
            author,
            created_at: 0,
            updated_at: None,
            read_time: 0,
        }
    }
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct User {
    pub id: Principal,
    pub name: Option<String>,
    pub bio: Option<String>,
    pub follower: u32,
    pub created_at: u64,
}

impl Storable for User {
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

impl User {
    pub fn new(id: Principal, created_at: u64) -> Self {
        Self {
            id,
            name: None,
            bio: None,
            follower: 0,
            created_at,
        }
    }
}

// candid Args section

#[derive(Debug, Clone, CandidType, Deserialize, Serialize, Default)]
pub struct SaveDraftArgs {
    pub title: Option<String>,
    pub content: Option<String>,
    pub detail: Option<StoryDetail>,
}
