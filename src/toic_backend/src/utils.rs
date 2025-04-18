/// Nanoseconds at 1 millisecond
pub const NANOS_IN_MILLIS: u64 = 1_000_000;

/// Gets current timestamp inside a canister, in milliseconds since the epoch (1970-01-01)
pub fn timestamp() -> u64 {
    ic_cdk::api::time() / NANOS_IN_MILLIS
}

#[cfg(test)]
pub mod mocks {
    use std::cell::{Cell, RefCell};

    use candid::Principal;

    thread_local! {
        static TIMESTAMP: Cell<u64> = Cell::new(0);
        static CALLER: RefCell<String> = RefCell::new("2chl6-4hpzw-vqaaa-aaaaa-c".to_string());
    }

    pub fn timestamp() -> u64 {
        let ts = TIMESTAMP.get();
        TIMESTAMP.with(|c| c.set(ts + 1));
        ts
    }

    pub fn caller() -> Principal {
        Principal::from_text(CALLER.with_borrow(|s| s.clone()).as_str()).unwrap()
    }

    pub fn reset_timestamp(time: u64) {
        TIMESTAMP.with(|c| c.set(time));
    }
}

#[cfg(test)]
mod tests {
    use super::timestamp;

    #[test]
    #[should_panic]
    fn timestamp_canister_only() {
        let _ = timestamp();
    }
}
