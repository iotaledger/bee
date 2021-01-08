// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{event::Bus, resource::ResHandle, worker::Worker};

use bee_storage::backend::StorageBackend;

use async_trait::async_trait;
use futures::{channel::oneshot, future::Future};

use std::any::Any;

#[async_trait]
pub trait Node: Send + Sized + 'static {
    type Builder: NodeBuilder<Self>;
    type Backend: StorageBackend;
    type Error: std::error::Error;

    async fn stop(mut self) -> Result<(), Self::Error>;

    fn spawn<W, G, F>(&mut self, g: G)
    where
        W: Worker<Self>,
        G: FnOnce(oneshot::Receiver<()>) -> F,
        F: Future<Output = ()> + Send + 'static;

    fn worker<W>(&self) -> Option<&W>
    where
        W: Worker<Self> + Send + Sync;

    fn register_resource<R: Any + Send + Sync>(&mut self, res: R);

    fn remove_resource<R: Any + Send + Sync>(&mut self) -> Option<R>;

    #[track_caller]
    fn resource<R: Any + Send + Sync>(&self) -> ResHandle<R>;

    #[track_caller]
    fn storage(&self) -> ResHandle<Self::Backend> {
        self.resource()
    }

    #[track_caller]
    fn bus(&self) -> ResHandle<Bus<'static>> {
        self.resource()
    }
}

#[async_trait(?Send)]
pub trait NodeBuilder<N: Node>: Sized {
    type Error: std::error::Error;
    type Config;

    fn new(config: Self::Config) -> Result<Self, Self::Error>;

    fn with_worker<W: Worker<N> + 'static>(self) -> Self
    where
        W::Config: Default;

    fn with_worker_cfg<W: Worker<N> + 'static>(self, config: W::Config) -> Self;

    fn with_resource<R: Any + Send + Sync>(self, res: R) -> Self;

    async fn finish(self) -> Result<N, Self::Error>;
}
