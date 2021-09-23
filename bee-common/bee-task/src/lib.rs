// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Task related utilities.

#![deny(missing_docs)]

use std::future::Future;

/// Instrumented task spawner with associated origin.
pub trait TaskSpawner {
    /// Origin of the task.
    const ORIGIN: &'static str;

    /// Spawns a task with or without instrumentation depending on a compile time feature.
    #[cfg_attr(feature = "tokio-console", track_caller)]
    fn spawn<F>(future: F) -> tokio::task::JoinHandle<<F as futures::Future>::Output>
    where
        F: Future + Send + 'static,
        <F as futures::Future>::Output: Send,
    {
        #[cfg(feature = "tokio-console")]
        {
            let caller = std::panic::Location::caller();
            let span = tracing::info_span!(
                target: "tokio::task",
                "task",
                origin = Self::ORIGIN,
                file = caller.file(),
                line = caller.line(),
            );

            tokio::spawn(tracing::Instrument::instrument(future, span))
        }

        #[cfg(not(feature = "tokio-console"))]
        tokio::spawn(future)
    }
}

/// Instrumented task spawner with a standalone origin.
pub struct StandaloneSpawner;

impl TaskSpawner for StandaloneSpawner {
    const ORIGIN: &'static str = "standalone";
}
