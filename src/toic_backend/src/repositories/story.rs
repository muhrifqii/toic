use candid::Principal;
use ic_stable_structures::{BTreeMap, Cell};
use lazy_static::lazy_static;
use std::{cell::RefCell, cmp::Reverse, sync::Arc, u64};

use crate::{
    memory::{
        ET_STORY_CONTENT_MEM_ID, ET_STORY_MEM_ID, IDX_STORY_AUTHOR_MEM_ID,
        IDX_STORY_CATEGORY_MEM_ID, MEMORY_MANAGER, SERIAL_STORY_MEM_ID,
    },
    structure::{
        AuditableRepository, BinaryTreeRepository, IndexRepository, IndexableRepository,
        Repository, SerialIdRepository,
    },
    types::{
        BTreeMapRefCell, Category, RepositoryError, RepositoryResult, SerialRefCell, SortBy,
        SortOrder, Story, StoryContent, SupportSize, VMemory,
    },
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

    static STORY_CONTENT: BTreeMapRefCell<u64, StoryContent> = RefCell::new(
        BTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(ET_STORY_CONTENT_MEM_ID))
        )
    );

    static STORY_CATEGORY_INDEX: BTreeMapRefCell<(Category, Reverse<u64>), ()> = RefCell::new(
        BTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(IDX_STORY_CATEGORY_MEM_ID))
        )
    );

    static STORY_AUTHOR_INDEX: BTreeMapRefCell<(Principal, Reverse<u64>), ()> = RefCell::new(
        BTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(IDX_STORY_AUTHOR_MEM_ID))
        )
    );
}

lazy_static! {
    pub static ref STORY_REPOSITORY: Arc<StoryRepository> = Arc::new(StoryRepository::default());
    pub static ref STORY_CONTENT_REPOSITORY: Arc<StoryContentRepository> =
        Arc::new(StoryContentRepository::default());
}

#[derive(Debug, Default)]
pub struct StoryCategoryIndexRepository;

impl IndexRepository<(Category, Reverse<u64>), u64, VMemory> for StoryCategoryIndexRepository {
    type Criteria = Category;
    type Cursor = u64;

    fn with_ref<F, R>(f: F) -> R
    where
        F: FnOnce(&RefCell<BTreeMap<(Category, Reverse<u64>), (), VMemory>>) -> R,
    {
        STORY_CATEGORY_INDEX.with(f)
    }

    fn find(
        &self,
        criteria: Self::Criteria,
        _: Option<SortOrder>,
        cursor: Option<Self::Cursor>,
        limit: usize,
    ) -> Vec<u64> {
        // default sort order is descending by Id (latest first)
        let until_id = cursor.map_or(u64::MAX, |c| c.saturating_sub(1));
        let start = (criteria, Reverse(until_id));
        let end = (criteria, Reverse(1));
        STORY_CATEGORY_INDEX.with_borrow(|m| {
            m.range(start..=end)
                .take(limit)
                .map(|((_, k), _)| k.0)
                .collect()
        })
    }
}

#[derive(Debug, Default)]
pub struct StoryAuthorIndexRepository;

impl IndexRepository<(Principal, Reverse<u64>), u64, VMemory> for StoryAuthorIndexRepository {
    type Criteria = Principal;
    type Cursor = u64;

    fn with_ref<F, R>(f: F) -> R
    where
        F: FnOnce(&RefCell<BTreeMap<(Principal, Reverse<u64>), (), VMemory>>) -> R,
    {
        STORY_AUTHOR_INDEX.with(f)
    }

    fn find(
        &self,
        criteria: Self::Criteria,
        _: Option<SortOrder>,
        cursor: Option<Self::Cursor>,
        limit: usize,
    ) -> Vec<u64> {
        // default sort order is descending by Id (latest first)
        let until_id = cursor.map_or(u64::MAX, |c| c.saturating_sub(1));
        let start = (criteria, Reverse(until_id));
        let end = (criteria, Reverse(1));
        STORY_AUTHOR_INDEX.with_borrow(|m| {
            m.range(start..=end)
                .take(limit)
                .map(|((_, k), _)| k.0)
                .collect()
        })
    }
}

#[derive(Debug, Default)]
pub struct StoryContentRepository;

impl BinaryTreeRepository<u64, StoryContent, VMemory> for StoryContentRepository {
    fn with_ref<F, R>(f: F) -> R
    where
        F: FnOnce(&RefCell<BTreeMap<u64, StoryContent, VMemory>>) -> R,
    {
        STORY_CONTENT.with(f)
    }
}

impl Repository<u64, StoryContent, VMemory> for StoryContentRepository {
    fn insert(&self, value: StoryContent) -> RepositoryResult<StoryContent> {
        if Self::with_ref(|cell| cell.borrow().contains_key(&value.id)) {
            return Err(RepositoryError::Conflict);
        }

        Self::with_ref(|cell| cell.borrow_mut().insert(value.id, value.clone()));
        Ok(value)
    }

    fn update(&self, value: StoryContent) -> RepositoryResult<StoryContent> {
        if Self::with_ref(|cell| !cell.borrow().contains_key(&value.id)) {
            return Err(RepositoryError::NotFound);
        }

        Self::with_ref(|cell| cell.borrow_mut().insert(value.id, value.clone()));
        Ok(value)
    }
}

#[derive(Debug, Default)]
pub struct StoryRepository {
    category_index: StoryCategoryIndexRepository,
    author_index: StoryAuthorIndexRepository,
}

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
    fn remove_indexes(&self, value: &Story) {
        self.category_index
            .remove(&(value.detail.category, Reverse(value.id)));
        self.author_index.remove(&(value.author, Reverse(value.id)));
    }

    fn add_indexes(&self, value: &Story) {
        self.category_index
            .insert((value.detail.category, Reverse(value.id)));
        self.author_index.insert((value.author, Reverse(value.id)));
    }

    fn clear_indexes(&self) {
        self.category_index.clear();
        self.author_index.clear();
    }
}

impl StoryRepository {
    pub fn get_stories_by_author(
        &self,
        author: Principal,
        cursor: Option<u64>,
        limit: usize,
    ) -> RepositoryResult<Vec<Story>> {
        let stories = self
            .author_index
            .find(author, None, cursor, limit)
            .into_iter()
            .filter_map(|id| self.get(&id))
            .collect();
        Ok(stories)
    }

    pub fn get_stories_by_categories(
        &self,
        categories: Vec<Category>,
        sort: SortOrder,
        cursor: Vec<Option<u64>>,
        limit: usize,
    ) -> RepositoryResult<Vec<Story>> {
        if !sort.is_sorted_by_id() {
            return Err(RepositoryError::UnsupportedOperation);
        }
        if limit < categories.len() || limit % categories.len() != 0 {
            // limit must be divisible by the number of categories
            // to ensure even distribution of stories across categories
            return Err(RepositoryError::IllegalArgument {
                reason: format!("limit size violation"),
            });
        }
        if categories.len() != cursor.len() {
            return Err(RepositoryError::IllegalArgument {
                reason: format!("categories and cursor must be of the same size"),
            });
        }

        let percategory_limit = limit / categories.len();
        let mut stories = Vec::new();

        for i in 0..categories.len() {
            let category = categories[i].clone();
            let cursor = cursor[i];
            let mut category_stories = self
                .category_index
                .find(category, None, cursor, percategory_limit)
                .into_iter()
                .filter_map(|id| self.get(&id))
                .collect::<Vec<_>>();
            stories.append(&mut category_stories);
        }

        Ok(stories)
    }
}
