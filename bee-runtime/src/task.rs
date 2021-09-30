// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Task utilities.

use std::future::Future;

/// Spawns task and tracks the calling location.
#[track_caller]
pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<<F as futures::Future>::Output>
where
    F: Future + Send + 'static,
    <F as futures::Future>::Output: Send,
{
    tokio::spawn(future)
}
