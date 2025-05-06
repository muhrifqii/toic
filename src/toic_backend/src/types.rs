use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::{
    storable::{Blob, Bound},
    BTreeMap, Cell, Storable,
};
use serde::{Deserialize, Serialize};
use strum::EnumString;
use thiserror::Error;

pub use crate::memory::VMemory;
use crate::token::Tokens;

pub type SerialRefCell = RefCell<Cell<u64, VMemory>>;
pub type BTreeMapRefCell<K, V> = RefCell<BTreeMap<K, V, VMemory>>;

pub type RepositoryResult<T> = Result<T, RepositoryError>;
pub type ServiceResult<T> = Result<T, ServiceError>;
pub type ApiResult<T> = Result<T, ErrorResponse>;

pub type SupportSize = u32;
pub type ViewSize = u32;
pub type Score = u64;

#[derive(Error, Debug, Eq, PartialEq, Clone)]
pub enum RepositoryError {
    #[error(r#"The requested entity was not found in the repository."#)]
    NotFound,
    #[error(r#"Cannot write on existing entity."#)]
    Conflict,
    #[error(r#"Invalid update operation: {reason}."#)]
    IllegalUpdate { reason: String },
    #[error("Unsupported operation")]
    UnsupportedOperation,
    #[error("Illegal argument: {reason}")]
    IllegalArgument { reason: String },
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
    #[error("Story not found")]
    StoryNotFound,
    #[error("Unprocessable entity: {reason}")]
    UnprocessableEntity { reason: String },
    #[error("{entity} already exists")]
    Conflict { entity: String },
    #[error("Transfer failed: {reason}")]
    TransferError { reason: String },
    #[error("{0}")]
    AiModelError(String),
}

pub trait AuditableEntity {
    fn id(&self) -> u64;
    fn set_id(&mut self, id: u64);
    fn set_created_at(&mut self, created_at: u64);
    fn set_updated_at(&mut self, updated_at: u64);
}

#[derive(
    Debug,
    Clone,
    CandidType,
    Deserialize,
    Serialize,
    EnumString,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Copy,
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

    // to allow tuple stable structure encoding
    const BOUND: Bound = Bound::Bounded {
        max_size: 24,
        is_fixed_size: true,
    };
}

#[derive(Debug, CandidType, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct StorablePrincipal(pub Principal);

impl Storable for StorablePrincipal {
    const BOUND: Bound = Blob::<29>::BOUND;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(
            Blob::<29>::try_from(self.0.as_slice())
                .expect("principal length should not exceed 29 bytes")
                .to_bytes()
                .into_owned(),
        )
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(Principal::from_slice(
            Blob::<29>::from_bytes(bytes).as_slice(),
        ))
    }
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
pub struct StoryContent {
    pub id: u64,
    pub content: String,
    pub author: Principal,
}

impl Storable for StoryContent {
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

impl StoryContent {
    pub fn new(id: u64, content: String, author: Principal) -> Self {
        Self {
            id,
            content,
            author,
        }
    }
}

#[derive(Debug, CandidType, Deserialize, Serialize, Clone)]
pub struct Story {
    pub id: u64,
    pub title: String,
    pub detail: StoryDetail,
    pub author: Principal,
    pub total_support: SupportSize,
    pub total_views: ViewSize,
    pub total_tip_support: Tokens,
    pub created_at: u64,
    pub updated_at: Option<u64>,
    pub read_time: u32,
    pub score: Score,
    pub author_name: Option<String>,
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

    fn set_id(&mut self, id: u64) {
        self.id = id
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
            author: draft.author,
            total_support: 0,
            total_views: 0,
            total_tip_support: 0_usize.into(),
            created_at: 0,
            updated_at: None,
            read_time: draft.read_time,
            score: 0,
            author_name: None,
        }
    }
}

#[derive(Debug, CandidType, Deserialize, Serialize, Clone)]
pub struct Draft {
    pub id: u64,
    pub title: String,
    pub detail: Option<StoryDetail>,
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

    fn set_id(&mut self, id: u64) {
        self.id = id
    }

    fn set_created_at(&mut self, created_at: u64) {
        self.created_at = created_at;
    }

    fn set_updated_at(&mut self, updated_at: u64) {
        self.updated_at = Some(updated_at);
    }
}

impl Draft {
    pub fn new(title: String, detail: Option<StoryDetail>, author: Principal) -> Self {
        Self {
            id: 0,
            title,
            detail,
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
    pub followed_categories: Vec<Category>,
    pub followed_authors: Vec<Principal>,
    pub onboarded: bool,
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
            followed_categories: vec![],
            followed_authors: vec![],
            created_at,
            onboarded: false,
        }
    }
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct Statistics {
    pub total_users: u32,
    pub total_stories: u32,
    pub total_drafts: u32,
    pub total_categories: u32,
    pub category_followers: Vec<(Category, u32)>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SupportGiven {
    pub support: SupportSize,
    pub token: Tokens,
}

impl SupportGiven {
    pub fn new(support: SupportSize, token: Tokens) -> Self {
        Self { support, token }
    }
}

impl Storable for SupportGiven {
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

#[derive(Debug, Clone, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub enum SortOrder {
    Asc(SortBy),
    Desc(SortBy),
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Asc(SortBy::default())
    }
}

impl SortOrder {
    pub fn is_asc(&self) -> bool {
        matches!(self, Self::Asc(_))
    }

    pub fn is_desc(&self) -> bool {
        matches!(self, Self::Desc(_))
    }

    pub fn is_sorted_by_id(&self) -> bool {
        matches!(self, Self::Asc(SortBy::Id) | Self::Desc(SortBy::Id))
    }
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize, Default, PartialEq, Eq)]
pub enum SortBy {
    #[default]
    Id,
    UpdatedAt,
}

// candid Args section

#[derive(Debug, Clone, CandidType, Deserialize, Serialize, Default)]
pub struct SaveDraftArgs {
    pub title: Option<String>,
    pub content: Option<String>,
    pub detail: Option<StoryDetail>,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize, Default)]
pub struct StoryInteractionArgs {
    pub id: u64,
    pub support: Option<SupportSize>,
    pub tip: Option<Tokens>,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize, Default)]
pub struct OnboardingArgs {
    pub name: Option<String>,
    pub bio: Option<String>,
    pub categories: Vec<Category>,
    pub referral_code: Option<String>,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum AssistActionArgs {
    ExpandWriting(u64),
    GenerateDescription(u64),
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct FetchStoriesArgs {
    pub category: Option<Category>,
    pub author: Option<Principal>,
    pub cursor: Option<u64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct FetchStoriesByScoreArgs {
    pub cursor: Option<(Score, u64)>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct UserOutline {
    pub id: Principal,
    pub name: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub message: String,
}
