// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{config::FullNodeConfig, FullNode, FullNodeError};

use crate::{
    config::{NetworkSpec, NodeConfigBuilder},
    core::{Core, CoreError, ResourceRegister, TopologicalOrder, WorkerStart, WorkerStop},
    plugins::{self, Mqtt, VersionChecker},
    shutdown,
    storage::StorageBackend,
    util, AUTOPEERING_VERSION, BEE_GIT_COMMIT, BEE_NAME, BEE_VERSION, PEERSTORE_PATH,
};

#[cfg(feature = "dashboard")]
use crate::plugins::Dashboard;

use bee_autopeering::{
    event::EventRx,
    peerstore::{InMemoryPeerStore, SledPeerStore, SledPeerStoreConfig},
    AutopeeringConfig, Local, NeighborValidator, ServiceProtocol, AUTOPEERING_SERVICE_NAME,
};
use bee_gossip::{Keypair, NetworkConfig, NetworkEventReceiver, Protocol};
use bee_ledger::workers::{pruning::config::PruningConfig, snapshot::config::SnapshotConfig};
use bee_protocol::workers::config::ProtocolConfig;
use bee_runtime::{
    event::Bus,
    node::{Node, NodeBuilder, NodeInfo},
    worker::Worker,
};
use bee_storage::system::StorageHealth;

use anymap::Map;
use async_trait::async_trait;
use futures::{channel::oneshot, future::Future};
use fxhash::FxBuildHasher;
use tokio::signal::unix::SignalKind;

use std::{
    any::{type_name, Any, TypeId},
    collections::{HashMap, HashSet},
    marker::PhantomData,
    pin::Pin,
};

pub struct FullNodeBuilder<B: StorageBackend> {
    config: FullNodeConfig<B>,
    deps: HashMap<TypeId, &'static [TypeId], FxBuildHasher>,
    worker_starts: HashMap<TypeId, Box<WorkerStart<FullNode<B>>>>,
    worker_stops: HashMap<TypeId, Box<WorkerStop<FullNode<B>>>>,
    worker_names: HashMap<TypeId, &'static str>,
    resource_registers: Vec<Box<ResourceRegister<FullNode<B>>>>,
}

impl<B: StorageBackend> FullNodeBuilder<B> {
    /// Returns the full node config.
    pub fn config(&self) -> &FullNodeConfig<B> {
        &self.config
    }

    /// Adds a plugin without config.
    pub fn with_plugin<P: plugins::Plugin>(self) -> Self
    where
        P::Config: Default,
    {
        self.with_worker::<plugins::PluginWorker<P>>()
    }

    /// Adds a plugin with config.
    pub fn with_plugin_cfg<P: plugins::Plugin>(self, config: P::Config) -> Self {
        self.with_worker_cfg::<plugins::PluginWorker<P>>(config)
    }
}

#[async_trait(?Send)]
impl<S: StorageBackend> NodeBuilder<FullNode<S>> for FullNodeBuilder<S> {
    type Error = FullNodeError;
    type Config = FullNodeConfig<S>;

    /// Creates a fullnode builder from the provided config.
    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            config,
            deps: HashMap::default(),
            worker_starts: HashMap::default(),
            worker_stops: HashMap::default(),
            worker_names: HashMap::default(),
            resource_registers: Vec::default(),
        })
    }

    /// Adds a worker (without config) to the node.
    fn with_worker<W: Worker<FullNode<S>> + 'static>(self) -> Self
    where
        W::Config: Default,
    {
        self.with_worker_cfg::<W>(W::Config::default())
    }

    /// Adds a worker (with config) to the node.
    fn with_worker_cfg<W: Worker<FullNode<S>> + 'static>(mut self, config: W::Config) -> Self {
        self.deps.insert(TypeId::of::<W>(), W::dependencies());
        self.worker_starts.insert(
            TypeId::of::<W>(),
            Box::new(|node| {
                Box::pin(async move {
                    log::debug!("Starting worker {}...", type_name::<W>());
                    match W::start(node, config).await {
                        Ok(w) => node.core.add_worker(w),
                        Err(e) => panic!("Worker `{}` failed to start: {:?}.", type_name::<W>(), e),
                    }
                })
            }),
        );
        self.worker_stops.insert(
            TypeId::of::<W>(),
            Box::new(|node| {
                Box::pin(async move {
                    match node.core.remove_worker::<W>().stop(node).await {
                        Ok(()) => {}
                        Err(e) => panic!("Worker `{}` failed to stop: {:?}.", type_name::<W>(), e),
                    }
                })
            }),
        );
        self.worker_names.insert(TypeId::of::<W>(), type_name::<W>());
        self
    }

    /// Adds a resource to the node.
    fn with_resource<R: Any + Send + Sync>(mut self, res: R) -> Self {
        self.resource_registers.push(Box::new(move |node| {
            node.register_resource(res);
        }));
        self
    }

    /// Finishes the builder, and either returns a full node instance, or an error explaining why it couldn't be built.
    async fn finish(self) -> Result<FullNode<S>, Self::Error> {
        let mut builder = self;

        // Print the network info the node is trying to connect to.
        let network_name = builder.config().network_spec().name();
        let network_id = builder.config().network_spec().id();
        let bech32_hrp = builder.config().network_spec().hrp();
        log::info!(
            "Joining network \"{}\"({}). Bech32 hrp \"{}\".",
            network_name,
            network_id,
            bech32_hrp
        );

        // Print the local entity data.
        log::info!("Local: {}", builder.config().local());

        // Add the resources that are shared throughout the node.
        let builder = add_node_resources(builder)?;

        // Initialize the gossip layer.
        let (gossip_rx, builder) = initialize_gossip_layer(builder).await?;

        // Initialize autopeering (if enabled).
        let (autopeering_rx, builder) = initialize_autopeering(builder).await?;

        // Initialize the ledger.
        let builder = initialize_ledger(builder);

        // Initialize the protocol.
        let builder = initialize_protocol(builder, gossip_rx, autopeering_rx);

        // Initialize the node API.
        let builder = initialize_api(builder).await;

        // Initialize the Tangle.
        let builder = initialize_tangle(builder);

        // Start the version checker.
        let builder = builder.with_worker::<VersionChecker>();

        // Start the MQTT broker.
        let mqtt_cfg = builder.config().mqtt.clone();
        let builder = builder.with_worker_cfg::<Mqtt>(mqtt_cfg);

        // Start serving the dashboard (if enabled).
        #[cfg(feature = "dashboard")]
        let builder = {
            let dashboard_cfg = builder.config().dashboard.clone();
            builder.with_worker_cfg::<Dashboard>(dashboard_cfg)
        };

        let FullNodeBuilder {
            config,
            deps,
            mut worker_starts,
            worker_stops,
            worker_names,
            resource_registers,
            ..
        } = builder;

        let worker_order = TopologicalOrder::sort(deps);

        let core = Core::new(worker_stops, worker_order, worker_names);

        let mut full_node = FullNode { config, core };

        for f in resource_registers {
            f(&mut full_node);
        }

        // Start all workers in topological order.
        for id in &full_node.core.worker_order.clone() {
            worker_starts.remove(id).unwrap()(&mut full_node).await;
        }

        log::info!("Initialized.");

        Ok(full_node)
    }
}

/// Creates and add the shared node resources.
///
/// Those are:
/// * the config
/// * the node info (name + version)
/// * the storage
/// * the event bus
/// * the shutdown signal receiver
fn add_node_resources<S: StorageBackend>(builder: FullNodeBuilder<S>) -> Result<FullNodeBuilder<S>, FullNodeError> {
    let config = builder.config().clone();

    let node_info = util::create_node_info();
    let storage_cfg = config.storage.clone();

    // TODO block ? Make new async ?
    let storage = S::start(storage_cfg).map_err(|e| CoreError::StorageBackend(Box::new(e)))?;

    if config.local().is_new() {
        storage
            .set_health(StorageHealth::Healthy)
            .map_err(|e| CoreError::StorageBackend(Box::new(e)))?;

        return Err(FullNodeError::InvalidOrNoIdentityPrivateKey(
            builder.config().local().encoded().to_string(),
        ));
    }

    let builder = builder
        .with_resource(config)
        .with_resource(node_info)
        .with_resource(storage)
        .with_resource(Bus::<TypeId>::default());

    #[cfg(unix)]
    let shutdown_rx = shutdown::shutdown_listener(vec![SignalKind::interrupt(), SignalKind::terminate()]);

    #[cfg(not(unix))]
    let shutdown_rx = shutdown::shutdown_listener();

    let builder = builder.with_resource(shutdown_rx);

    Ok(builder)
}

/// Initializes the gossip layer.
async fn initialize_gossip_layer<S: StorageBackend>(
    builder: FullNodeBuilder<S>,
) -> Result<(NetworkEventReceiver, FullNodeBuilder<S>), FullNodeError> {
    log::info!("Initializing gossip protocol...");

    let config = builder.config();

    let keypair = config.local().keypair().clone();
    let network_id = config.network_spec().id();
    let gossip_cfg = config.gossip.clone();

    let (builder, network_events) =
        bee_gossip::integrated::init::<FullNode<S>>(gossip_cfg, keypair, network_id, builder)
            .await
            .map_err(FullNodeError::GossipLayerInitialization)?;

    Ok((network_events, builder))
}

/// Initializes the ledger.
fn initialize_ledger<S: StorageBackend>(builder: FullNodeBuilder<S>) -> FullNodeBuilder<S> {
    log::info!("Initializing ledger...");

    let config = builder.config();

    let network_id = config.network_spec().id();
    let snapshot_cfg = config.snapshot.clone();
    let pruning_cfg = config.pruning.clone();

    bee_ledger::workers::init::<FullNode<S>>(builder, network_id, snapshot_cfg, pruning_cfg)
}

/// Initializes the protocol.
fn initialize_protocol<S: StorageBackend>(
    builder: FullNodeBuilder<S>,
    gossip_events: NetworkEventReceiver,
    autopeering_events: Option<bee_autopeering::event::EventRx>,
) -> FullNodeBuilder<S> {
    log::info!("Initializing protocol layer...");

    let config = builder.config();

    let NetworkSpec {
        name: network_name,
        id: network_id,
        hrp: _,
    } = config.network_spec().clone();

    let protocol_cfg = config.protocol.clone();

    bee_protocol::workers::init::<FullNode<S>>(
        protocol_cfg,
        (network_name, network_id),
        gossip_events,
        autopeering_events,
        builder,
    )
}

/// Initializes the (optional) autopeering service.
async fn initialize_autopeering<S: StorageBackend>(
    builder: FullNodeBuilder<S>,
) -> Result<(Option<bee_autopeering::event::EventRx>, FullNodeBuilder<S>), FullNodeError> {
    let config = builder.config();

    if !config.autopeering.enabled {
        Ok((None, builder))
    } else {
        log::info!("Initializing autopeering...");

        let autopeering_cfg = config.autopeering.clone();
        let network_name = config.network_spec().name().to_string();

        // The neighbor validator that includes/excludes certain peers by applying custom criteria.
        let neighbor_validator = FullNodeNeighborValidator {
            network_name: network_name.clone(),
        };

        // The peer store for persisting discovered peers.
        let peerstore_cfg = SledPeerStoreConfig::new().path(PEERSTORE_PATH);

        // A local entity that can sign outgoing messages, and announce services.
        let keypair = config.local().keypair().clone();
        let local = create_local_autopeering_entity(keypair, config);

        let quit_signal = tokio::signal::ctrl_c();

        let autopeering_rx = bee_autopeering::init::<SledPeerStore, _, _, _>(
            autopeering_cfg,
            AUTOPEERING_VERSION,
            network_name,
            local,
            peerstore_cfg,
            quit_signal,
            neighbor_validator,
        )
        .await
        .map_err(|e| FullNodeError::AutopeeringInitialization(e))?;

        Ok((Some(autopeering_rx), builder))
    }
}

/// Creates the local entity from a ED25519 keypair and a set of provided services.
fn create_local_autopeering_entity<S: StorageBackend>(
    keypair: Keypair,
    config: &FullNodeConfig<S>,
) -> bee_autopeering::Local {
    let local = bee_autopeering::Local::from_keypair(keypair);

    let mut write = local.write();

    // Announce the autopeering service.
    write.add_service(
        AUTOPEERING_SERVICE_NAME,
        ServiceProtocol::Udp,
        config.autopeering.bind_addr.port(),
    );

    // Announce the gossip service.
    // TODO: Make the bind address a SocketAddr instead of a Multiaddr
    let mut bind_addr = config.gossip.bind_multiaddr().clone();
    if let Some(Protocol::Tcp(port)) = bind_addr.pop() {
        write.add_service(config.network_spec().name(), ServiceProtocol::Tcp, port);
    } else {
        panic!("invalid gossip bind address");
    }

    drop(write);

    local
}

/// Initializes the API.
async fn initialize_api<S: StorageBackend>(builder: FullNodeBuilder<S>) -> FullNodeBuilder<S> {
    log::info!("Initializing REST API...");

    let config = builder.config();

    let NetworkSpec {
        name: network_name,
        id: network_id,
        hrp,
    } = config.network_spec().clone();

    let network_id = (network_name, network_id);
    let rest_api_cfg = config.rest_api.clone();
    let protocol_cfg = config.protocol.clone();

    let builder =
        bee_rest_api::endpoints::init::<FullNode<S>>(rest_api_cfg, protocol_cfg, network_id, hrp, builder).await;

    builder
}

/// Initializes the Tangle.
fn initialize_tangle<S: StorageBackend>(builder: FullNodeBuilder<S>) -> FullNodeBuilder<S> {
    log::info!("Initializing tangle...");

    let tangle_cfg = builder.config().tangle.clone();

    // TODO: `init` should probably just consume the config as any other crate does.
    bee_tangle::init::<FullNode<S>>(&tangle_cfg, builder)
}

#[derive(Clone)]
struct FullNodeNeighborValidator {
    network_name: String,
}

impl FullNodeNeighborValidator {
    pub fn new(network_name: String) -> Self {
        Self { network_name }
    }
}

impl NeighborValidator for FullNodeNeighborValidator {
    fn is_valid(&self, peer: &bee_autopeering::Peer) -> bool {
        peer.has_service(&self.network_name)
    }
}
