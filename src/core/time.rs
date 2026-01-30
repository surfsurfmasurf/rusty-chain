use std::time::{SystemTime, UNIX_EPOCH};

/// Current UNIX timestamp in milliseconds.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after unix epoch")
        .as_millis() as u64
}
