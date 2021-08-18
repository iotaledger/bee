// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "dashboard")]
use crate::plugins::Dashboard;

use crate::{
    config::NodeConfig,
    constants::{BEE_GIT_COMMIT, BEE_VERSION},
    node::{BeeNode, Error},
    plugins::{self, mqtt, VersionChecker},
    storage::StorageBackend,
};

use bee_runtime::{
    event::Bus,
    node::{Node, NodeBuilder, NodeInfo},
    worker::Worker,
};

use anymap::Map;
use async_trait::async_trait;
use futures::{channel::oneshot, future::Future};
use fxhash::FxBuildHasher;
use log::{debug, info, warn};

#[cfg(unix)]
use futures::future::select_all;
#[cfg(unix)]
use tokio::signal::unix::{signal, Signal, SignalKind};

use std::{
    any::{type_name, Any, TypeId},
    collections::{HashMap, HashSet},
    marker::PhantomData,
    pin::Pin,
};

type WorkerStart<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
type WorkerStop<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send;
type ResourceRegister<N> = dyn for<'a> FnOnce(&'a mut N);

fn shutdown_procedure(sender: oneshot::Sender<()>) {
    warn!("Gracefully shutting down the node, this may take some time.");

    if let Err(e) = sender.send(()) {
        panic!("Failed to send the shutdown signal: {:?}", e);
    }
}

#[cfg(unix)]
fn shutdown_listener(signals: Vec<SignalKind>) -> oneshot::Receiver<()> {
    let (sender, receiver) = oneshot::channel();

    tokio::spawn(async move {
        let mut signals = signals
            .iter()
            .map(|kind| signal(*kind).unwrap())
            .collect::<Vec<Signal>>();
        let signal_futures = signals.iter_mut().map(|signal| Box::pin(signal.recv()));

        let (signal_event, _, _) = select_all(signal_futures).await;

        if signal_event.is_none() {
            panic!("Shutdown signal stream failed, channel may have closed.");
        }

        shutdown_procedure(sender);
    });

    receiver
}

#[cfg(not(unix))]
fn shutdown_listener() -> oneshot::Receiver<()> {
    let (sender, receiver) = oneshot::channel();

    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            panic!("Failed to intercept CTRL-C: {:?}.", e);
        }

        shutdown_procedure(sender);
    });

    receiver
}

pub struct BeeNodeBuilder<B: StorageBackend> {
    deps: HashMap<TypeId, &'static [TypeId], FxBuildHasher>,
    worker_starts: HashMap<TypeId, Box<WorkerStart<BeeNode<B>>>>,
    worker_stops: HashMap<TypeId, Box<WorkerStop<BeeNode<B>>>>,
    worker_names: HashMap<TypeId, &'static str>,
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
        let version = if BEE_GIT_COMMIT.is_empty() {
            BEE_VERSION.to_owned()
        } else {
            BEE_VERSION.to_owned() + "-" + &BEE_GIT_COMMIT[0..7]
        };
        let node_info = NodeInfo {
            name: "Bee".to_owned(),
            version,
        };

        Ok(Self {
            deps: HashMap::default(),
            worker_starts: HashMap::default(),
            worker_stops: HashMap::default(),
            worker_names: HashMap::default(),
            resource_registers: Vec::default(),
            config: config.clone(),
        }
        .with_resource(node_info)
        // TODO block ? Make new async ?
        .with_resource((B::start(config.storage)).map_err(|e| Error::StorageBackend(Box::new(e)))?)
        .with_resource(Bus::<TypeId>::default()))
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
                    match node.remove_worker::<W>().stop(node).await {
                        Ok(()) => {}
                        Err(e) => panic!("Worker `{}` failed to stop: {:?}.", type_name::<W>(), e),
                    }
                })
            }),
        );
        self.worker_names.insert(TypeId::of::<W>(), type_name::<W>());
        self
    }

    fn with_resource<R: Any + Send + Sync>(mut self, res: R) -> Self {
        self.resource_registers.push(Box::new(move |node| {
            node.register_resource(res);
        }));
        self
    }

    async fn finish(mut self) -> Result<BeeNode<B>, Error> {
        if self.config.identity.2 {
            return Err(Error::InvalidOrNoIdentityPrivateKey(self.config.identity.1));
        }

        info!(
            "Joining network \"{}\"({}). Bech32 hrp \"{}\".",
            self.config.network_id.0, self.config.network_id.1, self.config.bech32_hrp
        );

        let config = self.config.clone();

        let network_config = config.network.clone();
        let network_id = config.network_id.1;

        let local_keys = config.identity.0.clone();

        let this = self.with_resource(config.clone()); // TODO: Remove clone

        info!("Initializing network layer...");
        let (this, events) = bee_network::integrated::init::<BeeNode<B>>(network_config, local_keys, network_id, this)
            .await
            .map_err(Error::NetworkInitializationFailed)?;

        #[cfg(unix)]
        let this = this.with_resource(shutdown_listener(vec![
            SignalKind::interrupt(),
            SignalKind::terminate(),
        ]));
        #[cfg(not(unix))]
        let this = this.with_resource(shutdown_listener());

        info!("Initializing ledger...");
        let this =
            bee_ledger::workers::init::<BeeNode<B>>(this, network_id, config.snapshot.clone(), config.pruning.clone());

        info!("Initializing protocol layer...");
        let this = bee_protocol::workers::init::<BeeNode<B>>(config.protocol.clone(), network_id, events, this);

        info!("Initializing REST API...");
        let this = bee_rest_api::endpoints::init::<BeeNode<B>>(
            config.rest_api.clone(),
            config.protocol.clone(),
            config.network_id.clone(),
            config.bech32_hrp.clone(),
            this,
        )
        .await;

        info!("Initializing tangle...");
        let this = bee_tangle::init::<BeeNode<B>>(&config.tangle, this);

        // MQTT core plugin
        info!("Initializing MQTT plugin...");
        let mqtt_config = config.mqtt.clone();
        let this = mqtt::init::<BeeNode<B>>(mqtt_config, this).await;

        let mut this = this.with_worker::<VersionChecker>();

        #[cfg(feature = "dashboard")]
        {
            this = this.with_worker_cfg::<Dashboard>(config.dashboard);
        }

        let mut node = BeeNode {
            workers: Map::new(),
            tasks: HashMap::new(),
            resources: Map::new(),
            worker_stops: this.worker_stops,
            worker_order: TopologicalOrder::sort(this.deps),
            worker_names: this.worker_names,
            phantom: PhantomData,
        };

        if true {
            let mut topological_order = "".to_owned();
            for worker_id in node.worker_order.iter() {
                // Unwrap is fine since worker_id is from the list of workers.
                let worker_name = node.worker_names.get(worker_id).unwrap().to_string();
                topological_order = format!("{} {}", topological_order, worker_name);
            }

            debug!("Workers topological order:{}", topological_order);
        }

        for f in this.resource_registers {
            f(&mut node);
        }

        for id in node.worker_order.clone() {
            this.worker_starts.remove(&id).unwrap()(&mut node).await;
        }

        info!("Initialized.");

        Ok(node)
    }
}

struct TopologicalOrder {
    graph: HashMap<TypeId, &'static [TypeId], FxBuildHasher>,
    non_visited: HashSet<TypeId, FxBuildHasher>,
    being_visited: HashSet<TypeId, FxBuildHasher>,
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

    fn sort(graph: HashMap<TypeId, &'static [TypeId], FxBuildHasher>) -> Vec<TypeId> {
        let non_visited = graph.keys().copied().collect();

        let mut this = Self {
            graph,
            non_visited,
            being_visited: HashSet::default(),
            order: vec![],
        };

        while let Some(&id) = this.non_visited.iter().next() {
            this.visit(id);
        }

        this.order
    }
}
