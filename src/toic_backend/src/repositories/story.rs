use ic_stable_structures::{BTreeMap, Cell};
use lazy_static::lazy_static;
use std::{cell::RefCell, sync::Arc};

use crate::{
    memory::{ET_STORY_MEM_ID, IDX_STORY_CATEGORY_MEM_ID, MEMORY_MANAGER, SERIAL_STORY_MEM_ID},
    structure::{
        AuditableRepository, BinaryTreeRepository, IndexableRepository, SerialIdRepository,
    },
    types::{BTreeMapRefCell, Category, SerialRefCell, Story, VMemory},
};

thread_local! {
    static NEXT_STORY_ID: SerialRefCell = RefCell::new(Cell::init(
        MEMORY_MANAGER.with_borrow(|m| m.get(SERIAL_STORY_MEM_ID)), 1
    ).expect("failed to init NEXT_STORY_ID"));

    static STORY: BTreeMapRefCell<u64, Story> = RefCell::new(
        BTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(ET_STORY_MEM_ID))
        )
    );

    static STORY_CATEGORY_INDEX: BTreeMapRefCell<(Category, u64), ()> = RefCell::new(
        BTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(IDX_STORY_CATEGORY_MEM_ID))
        )
    );
}

lazy_static! {
    pub static ref STORY_REPOSITORY: Arc<StoryRepository> = Arc::new(StoryRepository::default());
}

#[derive(Debug, Default)]
pub struct StoryRepository;

impl SerialIdRepository<VMemory> for StoryRepository {
    fn with_generator<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Cell<u64, VMemory>) -> R,
    {
        NEXT_STORY_ID.with_borrow_mut(f)
    }
}

impl BinaryTreeRepository<u64, Story, VMemory> for StoryRepository {
    fn with_ref<F, R>(f: F) -> R
    where
        F: FnOnce(&RefCell<BTreeMap<u64, Story, VMemory>>) -> R,
    {
        STORY.with(f)
    }
}

impl AuditableRepository<Story, VMemory> for StoryRepository {}

impl IndexableRepository<Story> for StoryRepository {
    fn remove_indexes(&self, value: &Story) {}

    fn add_indexes(&self, value: &Story) {}

    fn clear_indexes(&self) {}
}
