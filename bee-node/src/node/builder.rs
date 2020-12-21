// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::NodeConfig,
    node::BeeNode,
    plugins::{self, Mqtt},
    storage::Backend,
    version_checker::VersionCheckerWorker,
};

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_pt2::{
    event::Bus,
    node::{Node, NodeBuilder},
    worker::Worker,
};
use bee_network::{self, NetworkController, PeerId};
use bee_peering::{ManualPeerManager, PeerManager};
use bee_protocol::{events as protocol_events, init};
use bee_rest_api::config::RestApiConfig;

use anymap::Map;
use async_trait::async_trait;
use futures::{channel::oneshot, future::Future};
use log::{debug, info};
use thiserror::Error;
use tokio::sync::mpsc;

use std::{
    any::{type_name, Any, TypeId},
    collections::{HashMap, HashSet},
    marker::PhantomData,
    pin::Pin,
};

// TODO design proper type `PeerList`
type PeerList = HashMap<PeerId, (mpsc::UnboundedSender<Vec<u8>>, oneshot::Sender<()>)>;

type WorkerStart<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
type WorkerStop<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send;
type ResourceRegister<N> = dyn for<'a> FnOnce(&'a mut N);

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

#[derive(Error, Debug)]
pub enum Error {
    /// Occurs, when there is an error while reading the snapshot file.
    #[error("Reading snapshot file failed: {0}")]
    SnapshotError(bee_snapshot::Error),
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

    pub fn with_plugin<P: plugins::Plugin>(self) -> Self
    where
        P::Config: Default,
    {
        self.with_worker::<plugins::PluginWorker<P>>()
    }

    pub fn with_plugin_cfg<P: plugins::Plugin>(self, config: P::Config) -> Self {
        self.with_worker_cfg::<plugins::PluginWorker<P>>(config)
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
        let (this, events) = bee_network::init::<BeeNode<B>>(network_config, local_keys, network_id, this).await;

        let this = this.with_resource(ShutdownStream::new(ctrl_c_listener(), events));

        info!("Initializing snapshot handler...");
        let (this, snapshot) = bee_snapshot::init::<BeeNode<B>>(&config.snapshot, network_id, this)
            .await
            .map_err(Error::SnapshotError)?;

        // info!("Initializing ledger...");
        // let mut this = bee_ledger::init::<BeeNode<B>>(snapshot.header().ledger_index(), this);

        info!("Initializing protocol layer...");
        let this = init::<BeeNode<B>>(
            config.protocol.clone(),
            config.database.clone(),
            snapshot,
            network_id,
            this,
        );

        let mut this = this.with_worker::<VersionCheckerWorker>();
        this = this.with_worker_cfg::<Mqtt>(config.mqtt);

        info!("Initializing REST API...");
        let mut this = bee_rest_api::init::<BeeNode<B>>(
            RestApiConfig::build().finish(),
            config.protocol.clone(),
            config.network_id.clone(),
            this,
        )
        .await;
        // TODO: Read config from file

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
        let network = node.resource::<NetworkController>();
        let manual_peering_config = config.peering.manual.clone();
        ManualPeerManager::new(manual_peering_config, &network).await;

        // TODO we should probably remove this
        info!("Registering events...");
        protocol_events(&node, config.protocol.clone());

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
