use ic_stable_structures::Cell;
use std::cell::RefCell;

use crate::{
    structure::MEMORY_MANAGER,
    types::{SerialRefCell, SERIAL_DRAFT_ID, SERIAL_STORY_ID},
};

thread_local! {
    static NEXT_STORY_ID: SerialRefCell = RefCell::new(Cell::init(
        MEMORY_MANAGER.with_borrow(|m| m.get(SERIAL_STORY_ID)), 1
    ).expect("failed to init NEXT_STORY_ID"));

    static NEXT_DRAFT_ID: SerialRefCell = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(SERIAL_DRAFT_ID)), 1
        ).expect("failed to init NEXT_CONVERSATION_ID")
    );
}

#[derive(Debug, Default)]
pub struct StoryRepository;
