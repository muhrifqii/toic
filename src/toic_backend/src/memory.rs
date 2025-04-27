use std::cell::RefCell;

use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
};

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
}

pub type VMemory = VirtualMemory<DefaultMemoryImpl>;

pub const SERIAL_STORY_MEM_ID: MemoryId = MemoryId::new(1);
pub const SERIAL_DRAFT_MEM_ID: MemoryId = MemoryId::new(2);
pub const ET_STORY_MEM_ID: MemoryId = MemoryId::new(3);
pub const ET_DRAFT_MEM_ID: MemoryId = MemoryId::new(4);
pub const IDX_DRAFT_AUTHOR_MEM_ID: MemoryId = MemoryId::new(5);
pub const ET_USER_MEM_ID: MemoryId = MemoryId::new(6);
pub const IDX_STORY_CATEGORY_MEM_ID: MemoryId = MemoryId::new(7);
pub const IDX_STORY_AUTHOR_MEM_ID: MemoryId = MemoryId::new(8);
pub const ET_DRAFT_CONTENT_MEM_ID: MemoryId = MemoryId::new(9);
pub const ET_STORY_CONTENT_MEM_ID: MemoryId = MemoryId::new(10);
pub const IDX_STORY_SUPPORTER_MEM_ID: MemoryId = MemoryId::new(11);
pub const IDX_STORY_SCORING_MEM_ID: MemoryId = MemoryId::new(12);

pub const TOKEN_CONFIG_MEM_ID: MemoryId = MemoryId::new(13);
pub const TOKEN_TX_LOG_MEM_ID: MemoryId = MemoryId::new(14);
pub const TOKEN_ACCOUNT_BALANCE_MEM_ID: MemoryId = MemoryId::new(15);
