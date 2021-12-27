// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use tokio::time::{self, Duration};

use std::future::Future;

pub fn shutdown(secs: u64) -> impl Future + Send + Unpin {
    Box::new(Box::pin(time::sleep(Duration::from_secs(secs))))
}
