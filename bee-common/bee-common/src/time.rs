// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides common functions for timestamps.

/// Retrieves the current timestamp, including UTC offset.
pub fn now_local() -> time::OffsetDateTime {
    time::OffsetDateTime::try_now_local().expect("indeterminate utc offset")
}

/// Retrieves the current timestamp, at UTC.
pub fn now_utc() -> time::OffsetDateTime {
    time::OffsetDateTime::now_utc()
}

/// Creates a new time from a unix timestamp, at UTC.
pub fn from_unix_timestamp(timestamp: i64) -> time::OffsetDateTime {
    time::OffsetDateTime::from_unix_timestamp(timestamp)
}

/// Produces a formatted `String` from a timestamp, displayed as local time.
pub fn format(time: &time::OffsetDateTime) -> String {
    time.format("%F %T")
}
