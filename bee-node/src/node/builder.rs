// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "dashboard")]
use crate::plugins::Dashboard;

use crate::{
    config::NodeConfig,
    node::{BeeNode, Error},
    plugins::{self, Mqtt, VersionChecker},
    storage::StorageBackend,
};

use bee_network::{self, NetworkController};
use bee_peering::{ManualPeerManager, PeerManager};
use bee_runtime::{
    event::Bus,
    node::{Node, NodeBuilder},
    worker::Worker,
};

use anymap::Map;
use async_trait::async_trait;
use futures::{channel::oneshot, future::Future};
use log::{debug, info, warn};

use std::{
    any::{type_name, Any, TypeId},
    collections::{HashMap, HashSet},
    marker::PhantomData,
    pin::Pin,
};

type WorkerStart<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
type WorkerStop<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send;
type ResourceRegister<N> = dyn for<'a> FnOnce(&'a mut N);

fn ctrl_c_listener() -> oneshot::Receiver<()> {
    let (sender, receiver) = oneshot::channel();

    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            panic!("Failed to intercept CTRL-C: {:?}.", e);
        }

        warn!("Gracefully shutting down the node, this may take some time.");

        if let Err(e) = sender.send(()) {
            panic!("Failed to send the shutdown signal: {:?}.", e);
        }
    });

    receiver
}

pub struct BeeNodeBuilder<B: StorageBackend> {
    deps: HashMap<TypeId, &'static [TypeId]>,
    worker_starts: HashMap<TypeId, Box<WorkerStart<BeeNode<B>>>>,
    worker_stops: HashMap<TypeId, Box<WorkerStop<BeeNode<B>>>>,
    resource_registers: Vec<Box<ResourceRegister<BeeNode<B>>>>,
    config: NodeConfig<B>,
}

impl<B: StorageBackend> BeeNodeBuilder<B> {
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
impl<B: StorageBackend> NodeBuilder<BeeNode<B>> for BeeNodeBuilder<B> {
    type Error = Error;
    type Config = NodeConfig<B>;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            deps: HashMap::default(),
            worker_starts: HashMap::default(),
            worker_stops: HashMap::default(),
            resource_registers: Vec::default(),
            config: config.clone(),
        }
        .with_resource(Bus::<TypeId>::default())
        // TODO block ? Make new async ?
        .with_resource(
            futures::executor::block_on(B::start(config.storage)).map_err(|e| Error::StorageBackend(Box::new(e)))?,
        ))
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
        if self.config.peering.identity_private_key.2 {
            return Err(Error::InvalidOrNoIdentityPrivateKey(
                self.config.peering.identity_private_key.1,
            ));
        }

        info!(
            "Joining network {}({}).",
            self.config.network_id.0, self.config.network_id.1
        );

        let config = self.config.clone();

        let network_config = config.network.clone();
        let network_id = config.network_id.1;

        let max_unknown_peers = config.peering.manual.unknown_peers_limit;
        let local_keys = config.peering.identity_private_key.0.clone();

        let this = self.with_resource(config.clone()); // TODO: Remove clone

        info!("Initializing network layer...");
        let (this, events) =
            bee_network::init::<BeeNode<B>>(network_config, local_keys, network_id, max_unknown_peers, this).await;

        let this = this.with_resource(ctrl_c_listener());

        info!("Initializing snapshot handler...");
        let this = bee_snapshot::init::<BeeNode<B>>(&config.snapshot, network_id, this).await;

        info!("Initializing ledger...");
        let this = bee_ledger::init::<BeeNode<B>>(this);

        info!("Initializing protocol layer...");
        let this = bee_protocol::init::<BeeNode<B>>(config.protocol.clone(), network_id, events, this);

        info!("Initializing REST API...");
        let this = bee_rest_api::init::<BeeNode<B>>(
            config.rest_api.clone(),
            config.protocol.clone(),
            config.network_id.clone(),
            this,
        )
        .await;

        let mut this = this.with_worker::<VersionChecker>();
        this = this.with_worker_cfg::<Mqtt>(config.mqtt);
        #[cfg(feature = "dashboard")]
        {
            this =
                this.with_worker_cfg::<Dashboard>((config.dashboard, config.rest_api.clone(), config.peering.clone()));
        }

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
