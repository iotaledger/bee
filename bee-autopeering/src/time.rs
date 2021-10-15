// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) use tokio::time::interval;

pub(crate) fn unix_now() -> u64 {
    unix(SystemTime::now())
}

pub(crate) fn unix(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).expect("system clock error").as_secs()
}
