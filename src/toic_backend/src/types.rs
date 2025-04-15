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

pub const SERIAL_STORY_ID: MemoryId = MemoryId::new(1);
pub const SERIAL_DRAFT_ID: MemoryId = MemoryId::new(2);

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum ServiceError {
    #[error(r#"User identity {identity} cannot be found."#)]
    IdentityNotFound { identity: String },
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
