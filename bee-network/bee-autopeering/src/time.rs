// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) type Timestamp = u64;
pub(crate) type Timespan = u64;

/// Measured in seconds.
pub(crate) const SECOND: u64 = 1;
/// Measured in seconds.
pub(crate) const MINUTE: u64 = 60 * SECOND;
/// Measured in seconds.
pub(crate) const HOUR: u64 = 60 * MINUTE;

pub(crate) fn unix_now_secs() -> Timestamp {
    unix_time_secs(SystemTime::now())
}

pub(crate) fn unix_time_secs(t: SystemTime) -> Timestamp {
    // Panic: We don't allow faulty system clocks.
    t.duration_since(UNIX_EPOCH).expect("system clock error").as_secs()
}

pub(crate) fn since(past_ts: Timestamp) -> Option<Timespan> {
    delta(past_ts, unix_now_secs())
}

pub(crate) fn until(future_ts: Timestamp) -> Option<Timespan> {
    delta(unix_now_secs(), future_ts)
}

pub(crate) fn delta(older_ts: Timestamp, newer_ts: Timestamp) -> Option<Timespan> {
    newer_ts.checked_sub(older_ts)
}
