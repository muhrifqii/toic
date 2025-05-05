use std::cell::RefCell;

use ic_stable_structures::{BTreeMap, Cell, Memory, Storable};

use crate::{
    types::{AuditableEntity, RepositoryError, RepositoryResult, SortOrder},
    utils::timestamp,
};

pub trait SerialIdRepository<M>
where
    M: Memory,
{
    /// Wrap Serial Id data structure
    fn with_generator<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Cell<u64, M>) -> R;

    /// Peek the next value for the serial id
    fn peek_next_id(&self) -> u64 {
        Self::with_generator(|v| *v.get())
    }

    /// Get the next id and increment
    fn next_id(&self) -> u64 {
        Self::with_generator(|v| {
            let id = *v.get();
            v.set(id + 1).unwrap()
        })
    }
}

pub trait IndexableRepository<V>
where
    V: Clone + Storable,
{
    /// Removes the indexes for the current value
    fn remove_indexes(&self, value: &V);

    /// Adds the indexes for the current value
    fn add_indexes(&self, value: &V);

    /// Clears all the indexes
    fn clear_indexes(&self);

    /// Saves the indexes for the current value and removes the old indexes if
    /// the value has changed.
    fn save_indexes(&self, value: &V, old_value: Option<&V>) {
        if let Some(existing) = old_value {
            self.remove_indexes(existing);
        }
        self.add_indexes(value);
    }
}

pub trait BinaryTreeRepository<K, V, M>
where
    K: Clone + Ord + Storable,
    V: Clone + Storable,
    M: Memory,
{
    fn with_ref<F, R>(f: F) -> R
    where
        F: FnOnce(&RefCell<BTreeMap<K, V, M>>) -> R;

    fn get(&self, id: &K) -> Option<V> {
        Self::with_ref(|cell| cell.borrow().get(id))
    }

    fn get_all(&self) -> Vec<V> {
        Self::with_ref(|cell| cell.borrow().values().collect())
    }

    fn count(&self) -> u64 {
        Self::with_ref(|cell| cell.borrow().len())
    }

    fn exists(&self, id: &K) -> bool {
        Self::with_ref(|cell| cell.borrow().contains_key(id))
    }
}

pub trait Repository<K, V, M>: BinaryTreeRepository<K, V, M>
where
    K: Clone + Ord + Storable,
    V: Clone + Storable,
    M: Memory,
{
    fn insert(&self, value: V) -> RepositoryResult<V>;

    fn update(&self, value: V) -> RepositoryResult<V>;

    fn delete(&self, id: &K) -> RepositoryResult<K> {
        let old = Self::with_ref(|cell| cell.borrow_mut().remove(id));
        if old.is_some() {
            Ok(id.clone())
        } else {
            Err(RepositoryError::NotFound)
        }
    }
}

pub trait AuditableRepository<V, M>:
    BinaryTreeRepository<u64, V, M> + SerialIdRepository<M> + IndexableRepository<V>
where
    V: Clone + Storable + AuditableEntity,
    M: Memory,
{
    fn insert(&self, mut value: V) -> RepositoryResult<V> {
        if self.exists(&value.id()) {
            return Err(RepositoryError::Conflict);
        }

        let now = timestamp();
        let id = self.next_id();
        value.set_created_at(now);
        value.set_id(id);
        Self::with_ref(|cell| cell.borrow_mut().insert(id, value.clone()));
        self.save_indexes(&value, None);
        Ok(value)
    }

    fn update(&self, mut value: V) -> RepositoryResult<V> {
        if !self.exists(&value.id()) {
            return Err(RepositoryError::NotFound);
        }

        let now = timestamp();
        value.set_updated_at(now);
        let old_value = Self::with_ref(|cell| cell.borrow_mut().insert(value.id(), value.clone()));
        self.save_indexes(&value, old_value.as_ref());
        Ok(value)
    }

    fn delete(&self, id: &u64) -> RepositoryResult<u64> {
        let old = Self::with_ref(|cell| cell.borrow_mut().remove(id));
        if let Some(old_value) = old {
            self.remove_indexes(&old_value);
            Ok(id.clone())
        } else {
            Err(RepositoryError::NotFound)
        }
    }
}

pub trait IndexRepository<I, K, M>
where
    I: Clone + Ord + Storable,
    K: Clone + Storable,
    M: Memory,
{
    type Criteria;
    type Cursor;

    fn with_ref<F, R>(f: F) -> R
    where
        F: FnOnce(&RefCell<BTreeMap<I, (), M>>) -> R;

    /// Checks if an index exists.
    fn exists(&self, index: &I) -> bool {
        Self::with_ref(|cell| cell.borrow().get(index).is_some())
    }

    /// Inserts a new index.
    fn insert(&self, index: I) {
        Self::with_ref(|cell| {
            cell.borrow_mut().insert(index, ());
        });
    }

    /// Removes an index.
    fn remove(&self, index: &I) -> bool {
        Self::with_ref(|cell| cell.borrow_mut().remove(index).is_some())
    }

    /// Clears all indexes.
    fn clear(&self) {
        Self::with_ref(|cell| cell.borrow_mut().clear_new());
    }

    /// Finds entities based on criteria and cursor with a limit.
    fn find(
        &self,
        criteria: Self::Criteria,
        sort: Option<SortOrder>,
        cursor: Option<Self::Cursor>,
        limit: usize,
    ) -> Vec<K>;
}
