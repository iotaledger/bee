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

pub(crate) fn unix_time_secs(t: SystemTime) -> Timestamp {
    t.duration_since(UNIX_EPOCH).expect("system clock error").as_secs()
}

pub(crate) fn since(past_ts: Timestamp) -> Option<Timespan> {
    unix_now_secs().checked_sub(past_ts)
}

pub(crate) fn until(future_ts: Timestamp) -> Option<Timespan> {
    future_ts.checked_sub(unix_now_secs())
}

pub(crate) fn datetime_now() -> time::OffsetDateTime {
    time::OffsetDateTime::now_utc()
}

pub(crate) fn datetime(ts: Timestamp) -> time::OffsetDateTime {
    time::OffsetDateTime::from_unix_timestamp(ts as i64).expect("error creating datetime")
}
