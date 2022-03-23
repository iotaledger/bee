// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::future::Future;

use tokio::time::{self, Duration};

pub fn shutdown(secs: u64) -> impl Future + Send + Unpin {
    Box::new(Box::pin(time::sleep(Duration::from_secs(secs))))
}
