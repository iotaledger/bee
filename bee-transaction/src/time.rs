//! Module to create timestamps as used in IOTA transactions.

use std::time::{SystemTime, UNIX_EPOCH};

// NOTE: this will be changed u64
/// Returns current UNIX time.
pub fn get_unix_time_millis() -> i64 {
    let unix_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("error determining system time");
    (unix_time.as_secs() * 1000 + u64::from(unix_time.subsec_millis())) as i64
}
