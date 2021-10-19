// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) use tokio::time::sleep;

pub(crate) type Timestamp = u64;

pub(crate) fn unix_now_secs() -> Timestamp {
    unix_time_secs(SystemTime::now())
}

pub(crate) fn unix_time_secs(time: SystemTime) -> Timestamp {
    time.duration_since(UNIX_EPOCH).expect("system clock error").as_secs()
}
