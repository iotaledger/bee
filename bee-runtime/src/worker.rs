// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that deals with asynchronous workers in general.

use crate::node::Node;

use async_trait::async_trait;

use std::any::{Any, TypeId};

/// Errors that might occur during the lifetime of asynchronous workers.
#[derive(Debug)]
pub struct Error(pub Box<dyn std::error::Error + Send>);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Worker error: {:?}.", self.0)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

/// A trait representing a node worker.
///
/// Node workers are conceptually similar to actors in the actor programming model, but differ slightly in a number of
/// crucial ways.
///
/// - Workers may register and access shared state, known as 'resources'.
/// - Workers have a topological ordering that determine when they should be started and stopped.
#[async_trait]
pub trait Worker<N: Node>: Any + Send + Sync + Sized {
    /// The configuration state required to start this worker.
    type Config;
    /// An error that may be emitted during node startup and shutdown.
    type Error: std::error::Error;

    /// Generate a list of `TypeId`s representing the topological worker dependencies of this worker.
    ///
    /// Workers listed will be started before this worker and shut down after this worker.
    // TODO Replace with associated constant when stabilized.
    fn dependencies() -> &'static [TypeId] {
        &[]
    }

    /// Attempt to instantiate this worker with the given node and worker configuration.
    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error>;

    /// Attempt to stop an instance of this worker.
    async fn stop(self, _node: &mut N) -> Result<(), Self::Error> {
        Ok(())
    }
}
