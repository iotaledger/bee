// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
};

use async_trait::async_trait;
use bee_autopeering::{
    stores::{Options as RocksDbPeerStoreConfigOptions, RocksDbPeerStore, RocksDbPeerStoreConfig},
    NeighborValidator, ServiceProtocol, AUTOPEERING_SERVICE_NAME,
};
use bee_gossip::{Keypair, NetworkEventReceiver, Protocol};
use bee_metrics::Registry;
use bee_plugin_version_checker::VersionCheckerPlugin;
use bee_rest_api::endpoints::InitConfigFullNode;
use bee_runtime::{
    event::Bus,
    node::{Node, NodeBuilder},
    worker::Worker,
};
use bee_storage::system::StorageHealth;
use fxhash::FxBuildHasher;

use super::{config::FullNodeConfig, FullNode, FullNodeError};
use crate::{
    config::NetworkSpec,
    core::{Core, CoreError, ResourceRegister, TopologicalOrder, WorkerStart, WorkerStop},
    shutdown,
    storage::NodeStorageBackend,
    util,
    workers::MetricsRegistryWorker,
    AUTOPEERING_VERSION, BEE_VERSION,
};

/// A builder to create a Bee full node.
#[must_use]
pub struct FullNodeBuilder<S: NodeStorageBackend> {
    config: FullNodeConfig<S>,
    deps: HashMap<TypeId, &'static [TypeId], FxBuildHasher>,
    worker_starts: HashMap<TypeId, Box<WorkerStart<FullNode<S>>>>,
    worker_stops: HashMap<TypeId, Box<WorkerStop<FullNode<S>>>>,
    worker_names: HashMap<TypeId, &'static str>,
    resource_registers: Vec<Box<ResourceRegister<FullNode<S>>>>,
}

impl<S: NodeStorageBackend> FullNodeBuilder<S> {
    /// Returns the full node config.
    pub fn config(&self) -> &FullNodeConfig<S> {
        &self.config
    }

    /// Adds a plugin without config.
    pub fn with_plugin<P: bee_plugin::Plugin>(self) -> Self
    where
        P::Config: Default,
    {
        self.with_worker::<bee_plugin::PluginWorker<P>>()
    }

    /// Adds a plugin with config.
    pub fn with_plugin_cfg<P: bee_plugin::Plugin>(self, config: P::Config) -> Self {
        self.with_worker_cfg::<bee_plugin::PluginWorker<P>>(config)
    }
}

#[async_trait(?Send)]
impl<S: NodeStorageBackend> NodeBuilder<FullNode<S>> for FullNodeBuilder<S> {
    type Error = FullNodeError;
    type Config = FullNodeConfig<S>;

    /// Creates a full node builder from the provided config.
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

    /// Adds a worker (without config) to the full node.
    fn with_worker<W: Worker<FullNode<S>> + 'static>(self) -> Self
    where
        W::Config: Default,
    {
        self.with_worker_cfg::<W>(W::Config::default())
    }

    /// Adds a worker (with config) to the full node.
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

    /// Adds a resource to the full node.
    fn with_resource<R: Any + Send + Sync>(mut self, res: R) -> Self {
        self.resource_registers.push(Box::new(move |node| {
            node.register_resource(res);
        }));
        self
    }

    /// Returns the built full node.
    async fn finish(self) -> Result<FullNode<S>, Self::Error> {
        let builder = self;

        let network_spec = builder.config().network_spec();

        // Print the network info the node is trying to connect to.
        log::info!(
            "Joining network \"{}\", hrp \"{}\".",
            network_spec.name(),
            network_spec.hrp()
        );

        // Print the local entity data.
        log::info!("{}", builder.config().local());

        // Add the resources that are shared throughout the node.
        let builder = add_node_resources(builder)?;

        // Start the metrics registry worker.
        let metrics_cfg = builder.config.metrics.clone();
        let builder = builder.with_worker_cfg::<MetricsRegistryWorker>(metrics_cfg);

        // Initialize everything.
        let (gossip_rx, builder) = initialize_gossip_layer(builder)?;
        let (autopeering_rx, builder) = initialize_autopeering(builder).await?;
        let builder = initialize_ledger(builder);
        let builder = initialize_protocol(builder, gossip_rx, autopeering_rx);
        let builder = initialize_api(builder);
        let builder = initialize_tangle(builder);

        // Start the version checker.
        let builder = builder.with_worker_cfg::<VersionCheckerPlugin>(BEE_VERSION.to_string());

        // Start serving the dashboard (if enabled).
        #[cfg(feature = "dashboard")]
        let builder = initialize_dashboard(builder);

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
fn add_node_resources<S: NodeStorageBackend>(builder: FullNodeBuilder<S>) -> Result<FullNodeBuilder<S>, FullNodeError> {
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
        .with_resource(Bus::<TypeId>::default())
        .with_resource(Registry::default());

    #[cfg(unix)]
    let shutdown_rx = shutdown::shutdown_listener(vec![
        tokio::signal::unix::SignalKind::interrupt(),
        tokio::signal::unix::SignalKind::terminate(),
    ]);

    #[cfg(not(unix))]
    let shutdown_rx = shutdown::shutdown_listener();

    let builder = builder.with_resource(shutdown_rx);

    Ok(builder)
}

/// Initializes the gossip layer.
fn initialize_gossip_layer<S: NodeStorageBackend>(
    builder: FullNodeBuilder<S>,
) -> Result<(NetworkEventReceiver, FullNodeBuilder<S>), FullNodeError> {
    log::info!("Initializing gossip protocol...");

    let config = builder.config();

    let keypair = config.local().keypair().clone();
    let network_id = config.network_spec().id();
    let gossip_cfg = config.network.clone();

    let (builder, network_events) =
        bee_gossip::integrated::init::<FullNode<S>>(gossip_cfg, keypair, network_id, builder)
            .map_err(FullNodeError::GossipLayerInitialization)?;

    Ok((network_events, builder))
}

/// Initializes the ledger.
fn initialize_ledger<S: NodeStorageBackend>(builder: FullNodeBuilder<S>) -> FullNodeBuilder<S> {
    log::info!("Initializing ledger...");

    let config = builder.config();

    let network_id = config.network_spec().id();
    let snapshot_cfg = config.snapshot.clone();
    let pruning_cfg = config.pruning.clone();

    bee_ledger::workers::init::<FullNode<S>>(builder, network_id, snapshot_cfg, pruning_cfg)
}

/// Initializes the protocol.
fn initialize_protocol<S: NodeStorageBackend>(
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
async fn initialize_autopeering<S: NodeStorageBackend>(
    builder: FullNodeBuilder<S>,
) -> Result<(Option<bee_autopeering::event::EventRx>, FullNodeBuilder<S>), FullNodeError> {
    let config = builder.config();

    if !config.autopeering.enabled() {
        Ok((None, builder))
    } else {
        log::info!("Initializing autopeering...");

        let autopeering_cfg = config.autopeering.clone();
        let network_name = config.network_spec().name().to_string();

        // The neighbor validator that includes/excludes certain peers by applying custom criteria.
        let neighbor_validator = FullNodeNeighborValidator::new(network_name.clone());

        // The peer store for persisting discovered peers.
        let mut peerstore_options = RocksDbPeerStoreConfigOptions::default();
        peerstore_options.create_if_missing(true);
        peerstore_options.create_missing_column_families(true);
        let peerstore_cfg = RocksDbPeerStoreConfig::new(autopeering_cfg.peer_storage_path(), peerstore_options);

        // A local entity that can sign outgoing messages, and announce services.
        let keypair = config.local().keypair().clone();
        let local = create_local_autopeering_entity(keypair, config);

        let quit_signal = tokio::signal::ctrl_c();

        let autopeering_rx = bee_autopeering::init::<RocksDbPeerStore, _, _, _>(
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
fn create_local_autopeering_entity<S: NodeStorageBackend>(
    keypair: Keypair,
    config: &FullNodeConfig<S>,
) -> bee_autopeering::Local {
    let local = bee_autopeering::Local::from_keypair(keypair).expect("failed to create local entity");

    let port = if let Some(bind_addr) = config.autopeering.bind_addr_v4() {
        bind_addr.port()
    } else if let Some(bind_addr) = config.autopeering.bind_addr_v6() {
        bind_addr.port()
    } else {
        unreachable!("config validation ensures, that one bind address is available.");
    };

    // Announce the autopeering service.
    local.add_service(AUTOPEERING_SERVICE_NAME, ServiceProtocol::Udp, port);

    // Announce the gossip service.
    // TODO: Make the bind address a SocketAddr instead of a Multiaddr
    let mut bind_addr = config.network.bind_multiaddr().clone();
    if let Some(Protocol::Tcp(port)) = bind_addr.pop() {
        local.add_service(config.network_spec().name(), ServiceProtocol::Tcp, port);
    } else {
        panic!("invalid gossip bind address");
    }

    local
}

/// Initializes the API.
fn initialize_api<S: NodeStorageBackend>(builder: FullNodeBuilder<S>) -> FullNodeBuilder<S> {
    log::info!("Initializing REST API...");

    let config = builder.config();

    let NetworkSpec {
        name: network_name,
        id: network_id,
        hrp,
    } = config.network_spec().clone();

    let init_config = InitConfigFullNode {
        node_id: config.local.peer_id(),
        node_keypair: config.local.keypair().clone(),
        rest_api_config: config.rest_api.clone(),
        protocol_config: config.protocol.clone(),
        network_id: (network_name, network_id),
        bech32_hrp: hrp,
        #[cfg(feature = "dashboard")]
        dashboard_username: config.dashboard.auth().user().to_owned(),
    };

    bee_rest_api::endpoints::init_full_node::<FullNode<S>>(init_config, builder)
}

/// Initializes the Tangle.
fn initialize_tangle<S: NodeStorageBackend>(builder: FullNodeBuilder<S>) -> FullNodeBuilder<S> {
    log::info!("Initializing tangle...");

    let tangle_cfg = builder.config().tangle.clone();

    // TODO: `init` should probably just consume the config as any other crate does.
    bee_tangle::init::<FullNode<S>>(&tangle_cfg, builder)
}

#[cfg(feature = "dashboard")]
fn initialize_dashboard<S: NodeStorageBackend>(builder: FullNodeBuilder<S>) -> FullNodeBuilder<S> {
    log::info!("Initializing dashboard...");

    let config = builder.config();

    let dashboard_cfg = config.dashboard.clone();
    let rest_api_cfg = config.rest_api.clone();
    let node_id = config.local().peer_id();
    let node_keypair = config.local().keypair().clone();
    let node_alias = config.alias().clone();
    let bech32_hrp = config.network_spec().hrp().to_string();

    bee_plugin_dashboard::init::<FullNode<S>>(
        dashboard_cfg,
        rest_api_cfg,
        node_id,
        node_keypair,
        node_alias,
        bech32_hrp,
        builder,
    )
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
    fn is_valid<P: AsRef<bee_autopeering::Peer>>(&self, peer: P) -> bool {
        peer.as_ref().has_service(&self.network_name)
    }
}
