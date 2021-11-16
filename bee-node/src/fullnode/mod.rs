// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod builder;
pub mod config;

use self::{builder::FullNodeBuilder, config::FullNodeConfig};

use crate::{
    core::{Core, CoreError},
    shutdown::ShutdownRx,
    storage::StorageBackend,
};

use bee_runtime::{event::Bus, node::Node, resource::ResourceHandle, worker::Worker};

use async_trait::async_trait;
use futures::{channel::oneshot, future::Future};

use std::any::{type_name, Any, TypeId};

/// Full node related errors.
#[derive(Debug, thiserror::Error)]
pub enum FullNodeError {
    #[error(
        "Invalid or no identity private key provided. Add the newly generated {0} (or generate one with `bee p2p-identity`) to the configuration file and re-run the node."
    )]
    InvalidOrNoIdentityPrivateKey(String),
    #[error("Cause: {0}")]
    GossipLayerInitialization(#[from] bee_gossip::Error),
    #[error("Cause: {0}")]
    AutopeeringInitialization(Box<dyn std::error::Error>),
    #[error("{0}")]
    Core(#[from] CoreError),
}

/// Represents a Bee full node.
pub struct FullNode<S: StorageBackend> {
    pub(crate) config: FullNodeConfig<S>,
    pub(crate) core: Core<Self>,
}

impl<S: StorageBackend> FullNode<S> {
    /// Returns the node config.
    pub fn config(&self) -> &FullNodeConfig<S> {
        &self.config
    }

    /// Starts running the full node.
    pub async fn run(mut self) -> Result<(), FullNodeError> {
        log::info!("Full node running.");

        // Panic: unwrapping is fine because the builder added this resource.
        if let Err(e) = self.remove_resource::<ShutdownRx>().unwrap().await {
            log::warn!("awaiting shutdown failed: {:?}", e);
        }

        log::info!("Stopping full node...");

        self.stop().await.map_err(|_| CoreError::Shutdown)?;

        log::info!("Full node stopped.");

        Ok(())
    }
}

#[async_trait]
impl<S: StorageBackend> Node for FullNode<S> {
    type Builder = FullNodeBuilder<S>;
    type Backend = S;
    type Error = FullNodeError;

    fn register_resource<R: Any + Send + Sync>(&mut self, res: R) {
        self.core.resources.insert(ResourceHandle::new(res));
    }

    fn remove_resource<R: Any + Send + Sync>(&mut self) -> Option<R> {
        self.core.resources.remove::<ResourceHandle<R>>()?.try_unwrap()
    }

    #[track_caller]
    fn resource<R: Any + Send + Sync>(&self) -> ResourceHandle<R> {
        match self.core.resources.get::<ResourceHandle<R>>() {
            Some(res) => res.clone(),
            None => panic!("Unable to fetch node resource {}.", type_name::<R>(),),
        }
    }

    #[track_caller]
    fn spawn<W, G, F>(&mut self, g: G)
    where
        W: Worker<Self>,
        G: FnOnce(ShutdownRx) -> F,
        F: Future<Output = ()> + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let future = g(rx);

        let task = tokio::spawn(future);

        self.core
            .tasks
            .entry(TypeId::of::<W>())
            .or_default()
            .push((tx, Box::new(task)));
    }

    fn worker<W>(&self) -> Option<&W>
    where
        W: Worker<Self> + Send + Sync,
    {
        self.core.workers.get::<W>()
    }

    /// Stops the full node.
    async fn stop(mut self) -> Result<(), Self::Error> {
        for worker_id in self.core.worker_order.clone().into_iter().rev() {
            // Panic: unwrapping is fine since worker_id is from the list of workers.
            log::debug!("Stopping worker {}...", self.core.worker_names.get(&worker_id).unwrap());

            for (shutdown, task_fut) in self.core.tasks.remove(&worker_id).unwrap_or_default() {
                let _ = shutdown.send(());
                // TODO: Should we handle this error?
                let _ = task_fut.await;
            }

            // Panic: TODO unwrap
            self.core.worker_stops.remove(&worker_id).unwrap()(&mut self).await;
            self.resource::<Bus>().remove_listeners_by_id(worker_id);
        }

        // Panic: unwrapping is fine since the node register the backend itself.
        self.remove_resource::<Self::Backend>()
            .unwrap()
            .shutdown()
            .map_err(|e| CoreError::StorageBackend(Box::new(e)))?;

        Ok(())
    }
}
