// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Task utilities.

use std::future::Future;

/// Instrumented task spawner with associated origin.
pub trait TaskSpawner {
    /// Origin of the task.
    const ORIGIN: &'static str;

    /// Spawns a task with or without instrumentation depending on a compile time feature.
    #[track_caller]
    fn spawn<F>(future: F) -> tokio::task::JoinHandle<<F as futures::Future>::Output>
    where
        F: Future + Send + 'static,
        <F as futures::Future>::Output: Send,
    {
        tokio::spawn(future)
    }
}

/// Instrumented task spawner with a standalone origin.
pub struct StandaloneSpawner;

impl TaskSpawner for StandaloneSpawner {
    const ORIGIN: &'static str = "standalone";
}
