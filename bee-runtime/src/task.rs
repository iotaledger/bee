// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Task utilities.

use std::future::Future;

/// Spawns a task with or without instrumentation depending on a compile time feature.
#[track_caller]
pub fn task_spawn<F>(future: F) -> tokio::task::JoinHandle<<F as futures::Future>::Output>
where
    F: Future + Send + 'static,
    <F as futures::Future>::Output: Send,
{
    #[cfg(feature = "console")]
    {
        let caller = std::panic::Location::caller();
        let span = tracing::info_span!(
            target: "tokio::task",
            "task",
            file = caller.file(),
            line = caller.line(),
        );

        tokio::spawn(tracing::Instrument::instrument(future, span))
    }

    #[cfg(not(feature = "console"))]
    tokio::spawn(future)
}
