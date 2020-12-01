// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{config::NodeConfig, plugin, storage::Backend, version_checker::VersionCheckerWorker};

use bee_common::{
    event::Bus,
    node::{Node, NodeBuilder, ResHandle},
    shutdown,
    shutdown_stream::ShutdownStream,
    worker::Worker,
};
use bee_network::{self, Event, Multiaddr, Network, PeerId, ShortId};
use bee_peering::{ManualPeerManager, PeerManager};
use bee_protocol::Protocol;
use bee_rest_api::config::RestApiConfig;

use anymap::{any::Any as AnyMapAny, Map};
use async_trait::async_trait;
use futures::{
    channel::oneshot,
    future::Future,
    stream::{Fuse, StreamExt},
};
use log::{debug, info, trace, warn};
use thiserror::Error;
use tokio::spawn;

use std::{
    any::{type_name, Any, TypeId},
    collections::{HashMap, HashSet},
    marker::PhantomData,
    ops::Deref,
    pin::Pin,
};

type NetworkEventStream = ShutdownStream<Fuse<flume::r#async::RecvStream<'static, Event>>>;

// TODO design proper type `PeerList`
type PeerList = HashMap<PeerId, (flume::Sender<Vec<u8>>, oneshot::Sender<()>)>;

type WorkerStart<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
type WorkerStop<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send;
type ResourceRegister<N> = dyn for<'a> FnOnce(&'a mut N);

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
        let (receiver_tx, receiver_shutdown_tx) = Protocol::register(self.node, id.clone(), address).await;

        self.peers.insert(id, (receiver_tx, receiver_shutdown_tx));
    }

    #[inline]
    async fn peer_disconnected_handler(&mut self, id: PeerId) {
        // TODO unregister ?
        if let Some((_, shutdown)) = self.peers.remove(&id) {
            if let Err(e) = shutdown.send(()) {
                warn!("Sending shutdown to {} failed: {:?}.", id.short(), e);
            }
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

fn ctrl_c_listener() -> oneshot::Receiver<()> {
    let (sender, receiver) = oneshot::channel();

    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            panic!("Failed to intercept CTRL-C: {:?}.", e);
        }

        if let Err(e) = sender.send(()) {
            panic!("Failed to send the shutdown signal: {:?}.", e);
        }
    });

    receiver
}

#[async_trait]
impl<B: Backend> Node for BeeNode<B> {
    type Builder = BeeNodeBuilder<B>;
    type Backend = B;

    async fn stop(mut self) -> Result<(), shutdown::Error>
    where
        Self: Sized,
    {
        for worker_id in self.worker_order.clone().into_iter().rev() {
            for (shutdown, task_fut) in self.tasks.remove(&worker_id).unwrap_or_default() {
                let _ = shutdown.send(());
                // TODO: Should we handle this error?
                let _ = task_fut.await; //.map_err(|e| shutdown::Error::from(worker::Error(Box::new(e))))?;
            }
            self.worker_stops.remove(&worker_id).unwrap()(&mut self).await;
            self.resource::<Bus>().purge_worker_listeners(worker_id);
        }

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
        Self: Sized,
        W: Worker<Self>,
        G: FnOnce(oneshot::Receiver<()>) -> F,
        F: Future<Output = ()> + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();

        self.tasks
            .entry(TypeId::of::<W>())
            .or_default()
            .push((tx, Box::new(spawn(g(rx)))));
    }

    fn worker<W>(&self) -> Option<&W>
    where
        Self: Sized,
        W: Worker<Self> + Send + Sync,
    {
        self.workers.get::<W>()
    }
}

#[derive(Error, Debug)]
pub enum Error {
    /// Occurs, when there is an error while reading the snapshot file.
    #[error("Reading snapshot file failed.")]
    SnapshotError(bee_snapshot::Error),

    /// Occurs, when there is an error while shutting down the node.
    #[error("Shutting down failed.")]
    ShutdownError(#[from] bee_common::shutdown::Error),
}

pub struct BeeNodeBuilder<B: Backend> {
    deps: HashMap<TypeId, &'static [TypeId]>,
    worker_starts: HashMap<TypeId, Box<WorkerStart<BeeNode<B>>>>,
    worker_stops: HashMap<TypeId, Box<WorkerStop<BeeNode<B>>>>,
    resource_registers: Vec<Box<ResourceRegister<BeeNode<B>>>>,
    config: NodeConfig<B>,
}

impl<B: Backend> BeeNodeBuilder<B> {
    pub fn config(&self) -> &NodeConfig<B> {
        &self.config
    }

    pub fn with_plugin<P: plugin::Plugin>(self) -> Self
    where
        P::Config: Default,
    {
        self.with_worker::<plugin::PluginWorker<P>>()
    }

    pub fn with_plugin_cfg<P: plugin::Plugin>(self, config: P::Config) -> Self {
        self.with_worker_cfg::<plugin::PluginWorker<P>>(config)
    }
}

#[async_trait(?Send)]
impl<B: Backend> NodeBuilder<BeeNode<B>> for BeeNodeBuilder<B> {
    type Error = Error;
    type Config = NodeConfig<B>;

    fn new(config: Self::Config) -> Self {
        Self {
            deps: HashMap::default(),
            worker_starts: HashMap::default(),
            worker_stops: HashMap::default(),
            resource_registers: Vec::default(),
            config,
        }
        .with_resource(Bus::default())
    }

    fn with_worker<W: Worker<BeeNode<B>> + 'static>(self) -> Self
    where
        W::Config: Default,
    {
        self.with_worker_cfg::<W>(W::Config::default())
    }

    fn with_worker_cfg<W: Worker<BeeNode<B>> + 'static>(mut self, config: W::Config) -> Self {
        self.deps.insert(TypeId::of::<W>(), W::dependencies());
        self.worker_starts.insert(
            TypeId::of::<W>(),
            Box::new(|node| {
                Box::pin(async move {
                    debug!("Starting worker {}...", type_name::<W>());
                    match W::start(node, config).await {
                        Ok(w) => node.add_worker(w),
                        Err(e) => panic!("Worker `{}` failed to start: {:?}.", type_name::<W>(), e),
                    }
                })
            }),
        );
        self.worker_stops.insert(
            TypeId::of::<W>(),
            Box::new(|node| {
                Box::pin(async move {
                    debug!("Stopping worker {}...", type_name::<W>());
                    match node.remove_worker::<W>().stop(node).await {
                        Ok(()) => {}
                        Err(e) => panic!("Worker `{}` failed to stop: {:?}.", type_name::<W>(), e),
                    }
                })
            }),
        );
        self
    }

    fn with_resource<R: Any + Send + Sync>(mut self, res: R) -> Self {
        self.resource_registers.push(Box::new(move |node| {
            node.register_resource(res);
        }));
        self
    }

    async fn finish(mut self) -> Result<BeeNode<B>, Error> {
        info!(
            "Joining network {}({}).",
            self.config.network_id.0, self.config.network_id.1
        );

        let generated_new_local_keypair = self.config.peering.local_keypair.2;
        if generated_new_local_keypair {
            info!("Generated new local keypair: {}", self.config.peering.local_keypair.1);
            info!("Add this to your config, and restart the node.");
        }

        let config = self.config.clone();

        let network_config = config.network.clone();
        let network_id = config.network_id.1;
        let local_keys = config.peering.local_keypair.0.clone();

        let this = self
            .with_resource(config.clone()) // TODO: Remove clone
            .with_resource(PeerList::default());

        info!("Initializing network layer...");
        let (mut this, events) = bee_network::init::<BeeNode<B>>(network_config, local_keys, network_id, this).await;

        this = this.with_resource(ShutdownStream::new(ctrl_c_listener(), events.into_stream()));

        info!("Initializing snapshot handler...");
        let (this, snapshot) = bee_snapshot::init::<BeeNode<B>>(&config.snapshot, this)
            .await
            .map_err(Error::SnapshotError)?;

        // info!("Initializing ledger...");
        // node_builder = bee_ledger::whiteflag::init::<BeeNode<B>>(
        //     snapshot_metadata.index(),
        //     snapshot_state.into(),
        //     self.config.protocol.coordinator().clone(),
        //     node_builder,
        //     bus.clone(),
        // );

        info!("Initializing protocol layer...");
        let mut this = Protocol::init::<BeeNode<B>>(
            config.protocol.clone(),
            config.database.clone(),
            snapshot,
            network_id,
            this,
        );

        this = this.with_worker::<VersionCheckerWorker>();

        info!("Initializing REST API...");
        this = bee_rest_api::init::<BeeNode<B>>(RestApiConfig::build().finish(), config.network_id.clone(), this).await; // TODO: Read config from file

        let mut node = BeeNode {
            workers: Map::new(),
            tasks: HashMap::new(),
            resources: Map::new(),
            worker_stops: this.worker_stops,
            worker_order: TopologicalOrder::sort(this.deps),
            phantom: PhantomData,
        };

        for f in this.resource_registers {
            f(&mut node);
        }

        for id in node.worker_order.clone() {
            this.worker_starts.remove(&id).unwrap()(&mut node).await;
        }

        // TODO: turn into worker
        info!("Starting manual peer manager...");
        spawn({
            let network = node.resource::<Network>();
            let peering_manual = config.peering.manual.clone();
            async move {
                ManualPeerManager::new(peering_manual).run(&network).await;
            }
        });

        info!("Registering events...");
        bee_snapshot::events(&node);
        // bee_ledger::whiteflag::events(&bee_node, bus.clone());
        Protocol::events(&node, config.protocol.clone());

        info!("Initialized.");

        Ok(node)
    }
}

struct TopologicalOrder {
    graph: HashMap<TypeId, &'static [TypeId]>,
    non_visited: HashSet<TypeId>,
    being_visited: HashSet<TypeId>,
    order: Vec<TypeId>,
}

impl TopologicalOrder {
    fn visit(&mut self, id: TypeId) {
        if !self.non_visited.contains(&id) {
            return;
        }

        if !self.being_visited.insert(id) {
            panic!("Cyclic dependency detected.");
        }

        for &id in self.graph[&id] {
            self.visit(id);
        }

        self.being_visited.remove(&id);
        self.non_visited.remove(&id);
        self.order.push(id);
    }

    fn sort(graph: HashMap<TypeId, &'static [TypeId]>) -> Vec<TypeId> {
        let non_visited = graph.keys().copied().collect();

        let mut this = Self {
            graph,
            non_visited,
            being_visited: HashSet::new(),
            order: vec![],
        };

        while let Some(&id) = this.non_visited.iter().next() {
            this.visit(id);
        }

        this.order
    }
}
