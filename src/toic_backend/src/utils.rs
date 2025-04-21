/// Default reading speed in words per minute
pub const WPM: usize = 220;

/// Gets current timestamp inside a canister, in nanoseconds since the epoch (1970-01-01)
pub fn timestamp() -> u64 {
    ic_cdk::api::time()
}

/// Estimates the read time of a text based on the number of words and the reading speed in words per minute (WPM).
pub fn estimate_read_time(text: &str) -> u32 {
    let word_count = text.split_whitespace().count();
    let mut minutes = word_count / WPM;
    if word_count % WPM != 0 {
        minutes += 1; // round up
    }
    minutes.try_into().unwrap_or(0)
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
    use crate::utils::estimate_read_time;

    use super::timestamp;

    #[test]
    #[should_panic]
    fn timestamp_canister_only() {
        let _ = timestamp();
    }

    #[test]
    fn read_time() {
        let text = "This is a test text with several words.";
        let read_time = estimate_read_time(text);
        assert_eq!(read_time, 1); // 1 minute for less than 220 words
    }
}
