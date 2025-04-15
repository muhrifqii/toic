use std::cell::RefCell;

use ic_stable_structures::{memory_manager::MemoryManager, DefaultMemoryImpl, Memory, StableCell};

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
}

pub trait SerialIdRepository<M>
where
    M: Memory,
{
    /// Wrap Serial Id data structure
    fn with_generator<F, R>(f: F) -> R
    where
        F: FnOnce(&mut StableCell<u64, M>) -> R;

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
