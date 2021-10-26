// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) use tokio::time::sleep;

pub(crate) type Timestamp = u64;
pub(crate) type Timespan = u64;

pub(crate) const SECOND: u64 = 1;
pub(crate) const MINUTE: u64 = 60 * SECOND;
pub(crate) const HOUR: u64 = 60 * MINUTE;

pub(crate) fn unix_now_secs() -> Timestamp {
    unix_time_secs(SystemTime::now())
}

pub(crate) fn unix_time_secs(time: SystemTime) -> Timestamp {
    time.duration_since(UNIX_EPOCH).expect("system clock error").as_secs()
}

pub(crate) fn since(timestamp: Timestamp) -> Option<Timespan> {
    unix_now_secs().checked_sub(timestamp)
}

pub(crate) fn until(timestamp: Timestamp) -> Option<Timespan> {
    timestamp.checked_sub(unix_now_secs())
}

pub(crate) fn datetime_now() -> time::OffsetDateTime {
    time::OffsetDateTime::now_utc()
}

pub(crate) fn datetime(timestamp: Timestamp) -> time::OffsetDateTime {
    time::OffsetDateTime::from_unix_timestamp(timestamp as i64).expect("error creating datetime")
}
