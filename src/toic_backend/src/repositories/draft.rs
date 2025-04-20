use std::{cell::RefCell, sync::Arc, u64};

use candid::Principal;
use ic_stable_structures::{BTreeMap, Cell};
use lazy_static::lazy_static;

use crate::{
    memory::{ET_DRAFT_MEM_ID, IDX_DRAFT_AUTHOR_MEM_ID, MEMORY_MANAGER, SERIAL_DRAFT_MEM_ID},
    structure::{
        AuditableRepository, BinaryTreeRepository, IndexRepository, IndexableRepository,
        SerialIdRepository,
    },
    types::{BTreeMapRefCell, Draft, RepositoryResult, SerialRefCell, VMemory},
};

thread_local! {
    static NEXT_DRAFT_ID: SerialRefCell = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(SERIAL_DRAFT_MEM_ID)), 1
        ).expect("failed to init NEXT_CONVERSATION_ID")
    );

    static DRAFT: BTreeMapRefCell<u64, Draft> = RefCell::new(
        BTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(ET_DRAFT_MEM_ID))
        )
    );

    static DRAFT_INDEX: BTreeMapRefCell<(Principal, u64), ()> = RefCell::new(
        BTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(IDX_DRAFT_AUTHOR_MEM_ID))
        )
    );
}

lazy_static! {
    pub static ref DRAFT_REPOSITORY: Arc<DraftRepository> = Arc::new(DraftRepository::default());
}

#[derive(Debug, Default)]
pub struct DraftAuthorIndexRepository;

impl IndexRepository<(Principal, u64), u64, VMemory> for DraftAuthorIndexRepository {
    type Criteria = Principal;
    type Cursor = u64;

    fn with_ref<F, R>(f: F) -> R
    where
        F: FnOnce(&RefCell<BTreeMap<(Principal, u64), (), VMemory>>) -> R,
    {
        DRAFT_INDEX.with(f)
    }

    fn find(
        &self,
        criteria: Self::Criteria,
        cursor: Option<Self::Cursor>,
        limit: usize,
    ) -> Vec<u64> {
        let since_id = cursor.map_or(1, |c| c.saturating_add(1));
        let start = (criteria, since_id);
        let end = (criteria, u64::MAX);

        if limit == usize::default() {
            DRAFT_INDEX.with_borrow(|m| m.range(start..=end).map(|((_, k), _)| k).collect())
        } else {
            DRAFT_INDEX.with_borrow(|m| {
                m.range(start..=end)
                    .take(limit)
                    .map(|((_, k), _)| k)
                    .collect()
            })
        }
    }
}

#[derive(Debug, Default)]
pub struct DraftRepository {
    author_index: DraftAuthorIndexRepository,
}

impl SerialIdRepository<VMemory> for DraftRepository {
    fn with_generator<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Cell<u64, VMemory>) -> R,
    {
        NEXT_DRAFT_ID.with_borrow_mut(f)
    }
}

impl BinaryTreeRepository<u64, Draft, VMemory> for DraftRepository {
    fn with_ref<F, R>(f: F) -> R
    where
        F: FnOnce(&RefCell<BTreeMap<u64, Draft, VMemory>>) -> R,
    {
        DRAFT.with(f)
    }
}

impl AuditableRepository<Draft, VMemory> for DraftRepository {}

impl IndexableRepository<Draft> for DraftRepository {
    fn remove_indexes(&self, value: &Draft) {
        self.author_index.remove(&(value.author, value.id));
    }

    fn add_indexes(&self, value: &Draft) {
        self.author_index.insert((value.author, value.id));
    }

    fn clear_indexes(&self) {
        self.author_index.clear();
    }
}

impl DraftRepository {
    pub fn get_drafts_by_author(
        &self,
        author: Principal,
        cursor: Option<u64>,
        limit: usize,
    ) -> RepositoryResult<Vec<Draft>> {
        let draft_ids = self.author_index.find(author, cursor, limit);

        let drafts = draft_ids
            .into_iter()
            .filter_map(|id| self.get(&id))
            .collect();

        Ok(drafts)
    }
}
