// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod builder;
mod error;

pub use builder::BeeNodeBuilder;
pub use error::Error;

use crate::{config::NodeConfig, storage::Backend};

use bee_common::{event::Bus, shutdown_stream::ShutdownStream};
use bee_common_pt2::{
    node::{Node, ResHandle},
    worker::Worker,
};
use bee_network::{self, Event, Multiaddr, NetworkListener, PeerId, ShortId};
use bee_protocol::{register, unregister};

use anymap::{any::Any as AnyMapAny, Map};
use async_trait::async_trait;
use futures::{
    channel::oneshot,
    future::Future,
    stream::{Fuse, StreamExt},
};
use log::{info, trace, warn};
use tokio::sync::mpsc;

use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    ops::Deref,
    pin::Pin,
};

type NetworkEventStream = ShutdownStream<Fuse<NetworkListener>>;

// TODO design proper type `PeerList`
type PeerList = HashMap<PeerId, (mpsc::UnboundedSender<Vec<u8>>, oneshot::Sender<()>)>;

type WorkerStop<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send;

#[allow(clippy::type_complexity)]
pub struct BeeNode<B> {
    workers: Map<dyn AnyMapAny + Send + Sync>,
    tasks: HashMap<
        TypeId,
        Vec<(
            oneshot::Sender<()>,
            // TODO Result ?
            Box<dyn Future<Output = Result<(), tokio::task::JoinError>> + Send + Sync + Unpin>,
        )>,
    >,
    resources: Map<dyn AnyMapAny + Send + Sync>,
    worker_stops: HashMap<TypeId, Box<WorkerStop<Self>>>,
    worker_order: Vec<TypeId>,
    phantom: PhantomData<B>,
}

impl<B: Backend> BeeNode<B> {
    fn add_worker<W: Worker<Self> + Send + Sync>(&mut self, worker: W) {
        self.workers.insert(worker);
    }

    fn remove_worker<W: Worker<Self> + Send + Sync>(&mut self) -> W {
        self.workers
            .remove()
            .unwrap_or_else(|| panic!("Failed to remove worker `{}`", type_name::<W>()))
    }

    pub fn config(&self) -> impl Deref<Target = NodeConfig<B>> + Clone {
        self.resource()
    }

    #[allow(missing_docs)]
    pub async fn run(mut self) -> Result<(), Error> {
        info!("Running.");

        let mut network_events_stream = self.remove_resource::<NetworkEventStream>().unwrap();

        let mut runtime = NodeRuntime {
            peers: PeerList::default(),
            node: &self,
        };

        while let Some(event) = network_events_stream.next().await {
            trace!("Received event {:?}.", event);

            runtime.process_event(event).await;
        }

        info!("Stopping...");

        for (_, (_, shutdown)) in runtime.peers.into_iter() {
            // TODO: Should we handle this error?
            let _ = shutdown.send(());
        }

        self.stop().await.expect("Failed to properly stop node");

        info!("Stopped.");

        Ok(())
    }
}

#[async_trait]
impl<B: Backend> Node for BeeNode<B> {
    type Builder = BeeNodeBuilder<B>;
    type Backend = B;
    type Error = Error;

    async fn stop(mut self) -> Result<(), Self::Error> {
        for worker_id in self.worker_order.clone().into_iter().rev() {
            for (shutdown, task_fut) in self.tasks.remove(&worker_id).unwrap_or_default() {
                let _ = shutdown.send(());
                // TODO: Should we handle this error?
                let _ = task_fut.await; //.map_err(|e| shutdown::Error::from(worker::Error(Box::new(e))))?;
            }
            self.worker_stops.remove(&worker_id).unwrap()(&mut self).await;
            self.resource::<Bus>().remove_listeners_by_id(worker_id);
        }

        // Unwrapping is fine since the node register the backend itself.
        self.remove_resource::<B>()
            .unwrap()
            .shutdown()
            .await
            .map_err(Error::StorageBackend)?;

        Ok(())
    }

    fn register_resource<R: Any + Send + Sync>(&mut self, res: R) {
        self.resources.insert(ResHandle::new(res));
    }

    fn remove_resource<R: Any + Send + Sync>(&mut self) -> Option<R> {
        self.resources.remove::<ResHandle<R>>()?.try_unwrap()
    }

    #[track_caller]
    fn resource<R: Any + Send + Sync>(&self) -> ResHandle<R> {
        self.resources
            .get::<ResHandle<R>>()
            .unwrap_or_else(|| panic!("Unable to fetch node resource {}", type_name::<R>()))
            .clone()
    }

    fn spawn<W, G, F>(&mut self, g: G)
    where
        W: Worker<Self>,
        G: FnOnce(oneshot::Receiver<()>) -> F,
        F: Future<Output = ()> + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();

        self.tasks
            .entry(TypeId::of::<W>())
            .or_default()
            .push((tx, Box::new(tokio::spawn(g(rx)))));
    }

    fn worker<W>(&self) -> Option<&W>
    where
        W: Worker<Self> + Send + Sync,
    {
        self.workers.get::<W>()
    }
}

struct NodeRuntime<'a, B: Backend> {
    peers: PeerList,
    node: &'a BeeNode<B>,
}

impl<'a, B: Backend> NodeRuntime<'a, B> {
    #[inline]
    async fn process_event(&mut self, event: Event) {
        match event {
            Event::PeerAdded { id } => self.peer_added_handler(id).await,
            Event::PeerRemoved { id } => self.peer_removed_handler(id).await,
            Event::PeerConnected { id, address } => self.peer_connected_handler(id, address).await,
            Event::PeerDisconnected { id } => self.peer_disconnected_handler(id).await,
            Event::MessageReceived { message, from } => self.peer_message_received_handler(message, from).await,
            _ => (), // Ignore all other events for now
        }
    }

    #[inline]
    async fn peer_added_handler(&mut self, id: PeerId) {
        info!("Added peer: {}", id.short());
    }

    #[inline]
    async fn peer_removed_handler(&mut self, id: PeerId) {
        info!("Removed peer: {}", id.short());
    }

    #[inline]
    async fn peer_connected_handler(&mut self, id: PeerId, address: Multiaddr) {
        let (receiver_tx, receiver_shutdown_tx) = register(self.node, id.clone(), address).await;

        self.peers.insert(id, (receiver_tx, receiver_shutdown_tx));
    }

    #[inline]
    async fn peer_disconnected_handler(&mut self, id: PeerId) {
        if let Some((_, shutdown)) = self.peers.remove(&id) {
            if let Err(e) = shutdown.send(()) {
                warn!("Sending shutdown to {} failed: {:?}.", id.short(), e);
            }
            unregister(self.node, id).await;
        }
    }

    #[inline]
    async fn peer_message_received_handler(&mut self, message: Vec<u8>, from: PeerId) {
        if let Some(peer) = self.peers.get_mut(&from) {
            if let Err(e) = peer.0.send(message) {
                warn!("Sending PeerWorkerEvent::Message to {} failed: {}.", from.short(), e);
            }
        }
    }
}
