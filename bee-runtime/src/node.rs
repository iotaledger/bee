// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Traits used to represent bee nodes and allow for their instantiation.

use crate::{event::Bus, resource::ResourceHandle, worker::Worker};

use bee_storage::backend::StorageBackend;

use async_trait::async_trait;
use futures::{channel::oneshot, future::Future};

use std::any::Any;

/// A type holding information about a node.
pub struct NodeInfo {
    /// Name of the node.
    pub name: String,
    /// Version of the node.
    pub version: String,
}

/// A trait representing a node framework through which node workers may communicate.
#[async_trait]
pub trait Node: Send + Sized + 'static {
    /// The builder type used to create instances of this node.
    type Builder: NodeBuilder<Self>;
    /// The storage backend used by this node.
    type Backend: StorageBackend;
    /// The type of errors that may be emitted as a result of the build process.
    type Error: std::error::Error;

    /// Stop the node, ending the execution of all workers in a timely manner.
    async fn stop(mut self) -> Result<(), Self::Error>;

    /// Spawn a new node task associated with the given worker.
    ///
    /// The task will be shut down with the worker to preserve topological worker ordering.
    fn spawn<W, G, F>(&mut self, g: G)
    where
        W: Worker<Self>,
        G: FnOnce(oneshot::Receiver<()>) -> F,
        F: Future<Output = ()> + Send + 'static;

    /// Get a reference to the state of a worker.
    fn worker<W>(&self) -> Option<&W>
    where
        W: Worker<Self> + Send + Sync;

    /// Register a new resource with the node such that other workers may access it via [`Node::resource`].
    fn register_resource<R: Any + Send + Sync>(&mut self, res: R);

    /// Attempt to remove a resource from the node, returning `None` if no such resource was registered with the node.
    fn remove_resource<R: Any + Send + Sync>(&mut self) -> Option<R>;

    /// Obtain an owning handle to a node resource.
    #[track_caller]
    fn resource<R: Any + Send + Sync>(&self) -> ResourceHandle<R>;

    /// Obtain an owning handle to the node's info.
    #[track_caller]
    fn info(&self) -> ResourceHandle<NodeInfo> {
        self.resource()
    }

    /// Obtain an owning handle to the node's storage backend.
    #[track_caller]
    fn storage(&self) -> ResourceHandle<Self::Backend> {
        self.resource()
    }

    /// Obtain an owning handle to the node's event bus.
    #[track_caller]
    fn bus(&self) -> ResourceHandle<Bus<'static>> {
        self.resource()
    }
}

/// A trait that provides generic build configuration capabilities for a node.
#[async_trait(?Send)]
pub trait NodeBuilder<N: Node>: Sized {
    /// The type of errors that may be emitted as a result of the build process.
    type Error: std::error::Error;
    /// Global configuration provided to the node on creation.
    type Config;

    /// Begin building a new node with the provided configuration state.
    fn new(config: Self::Config) -> Result<Self, Self::Error>;

    /// Register a worker, with default configuration state, that should be started with the node.
    fn with_worker<W: Worker<N> + 'static>(self) -> Self
    where
        W::Config: Default;

    /// Register a worker, with the given configuration state, that should be started with the node.
    fn with_worker_cfg<W: Worker<N> + 'static>(self, config: W::Config) -> Self;

    /// Provide a resource that should be registered with the node upon start.
    fn with_resource<R: Any + Send + Sync>(self, res: R) -> Self;

    /// Finish building the node, returning the final node.
    async fn finish(self) -> Result<N, Self::Error>;
}
