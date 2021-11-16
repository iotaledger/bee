// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{config::EntryNodeConfig, EntryNode, EntryNodeError};

use crate::{
    core::{Core, ResourceRegister, TopologicalOrder, WorkerStart, WorkerStop},
    plugins::VersionChecker,
    shutdown, util, AUTOPEERING_VERSION, PEERSTORE_PATH,
};

use bee_autopeering::{
    peerstore::{SledPeerStore, SledPeerStoreConfig},
    NeighborValidator, ServiceProtocol, AUTOPEERING_SERVICE_NAME,
};
use bee_gossip::Keypair;
use bee_runtime::{
    event::Bus,
    node::{Node, NodeBuilder},
    shutdown_stream::ShutdownStream,
    worker::Worker,
};

use async_trait::async_trait;
use futures::StreamExt;
use fxhash::FxBuildHasher;
use tokio::signal::unix::SignalKind;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    convert::Infallible,
};

/// A builder to create a Bee entry node (autopeering).
pub struct EntryNodeBuilder {
    config: EntryNodeConfig,
    deps: HashMap<TypeId, &'static [TypeId], FxBuildHasher>,
    worker_starts: HashMap<TypeId, Box<WorkerStart<EntryNode>>>,
    worker_stops: HashMap<TypeId, Box<WorkerStop<EntryNode>>>,
    worker_names: HashMap<TypeId, &'static str>,
    resource_registers: Vec<Box<ResourceRegister<EntryNode>>>,
}

impl EntryNodeBuilder {
    /// Returns the node config.
    pub(crate) fn config(&self) -> &EntryNodeConfig {
        &self.config
    }
}

#[async_trait(?Send)]
impl NodeBuilder<EntryNode> for EntryNodeBuilder {
    type Error = EntryNodeError;
    type Config = EntryNodeConfig;

    /// Creates an entry node builder from the provided config.
    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            config: config.clone(),
            deps: HashMap::default(),
            worker_starts: HashMap::default(),
            worker_stops: HashMap::default(),
            worker_names: HashMap::default(),
            resource_registers: Vec::default(),
        })
    }

    /// Adds a worker (without config) to the entry node builder.
    fn with_worker<W: Worker<EntryNode> + 'static>(self) -> Self
    where
        W::Config: Default,
    {
        self.with_worker_cfg::<W>(W::Config::default())
    }

    /// Adds a worker (with config) to the entry node builder.
    fn with_worker_cfg<W: Worker<EntryNode> + 'static>(mut self, config: W::Config) -> Self {
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

    /// Adds a resource to the entry node builder.
    fn with_resource<R: Any + Send + Sync>(mut self, res: R) -> Self {
        self.resource_registers.push(Box::new(move |node| {
            node.register_resource(res);
        }));
        self
    }

    /// Returns the built entry node.
    async fn finish(mut self) -> Result<EntryNode, Self::Error> {
        let builder = self;

        if !builder.config().autopeering.enabled {
            return Err(EntryNodeError::DisabledAutopeering);
        }

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

        // Initialize autopeering (if enabled).
        let (autopeering_rx, builder) = initialize_autopeering(builder).await?;

        // Start the version checker.
        let builder = builder.with_worker::<VersionChecker>();

        // Start the autopeering event logger.
        let builder = builder.with_worker_cfg::<AutopeeringEventLogger>(autopeering_rx);

        let EntryNodeBuilder {
            config,
            deps,
            mut worker_starts,
            worker_stops,
            worker_names,
            resource_registers,
        } = builder;

        let worker_order = TopologicalOrder::sort(deps);

        let core = Core::new(worker_stops, worker_order, worker_names);

        let mut entry_node = EntryNode { config, core };

        // Add all resources.
        for f in resource_registers {
            f(&mut entry_node);
        }

        // Start all workers in topological order.
        for id in &entry_node.core.worker_order.clone() {
            worker_starts.remove(id).unwrap()(&mut entry_node).await;
        }

        log::info!("Initialized.");

        Ok(entry_node)
    }
}

/// Creates and add the shared node resources.
///
/// Those are:
/// * the config
/// * the node info (name + version)
/// * the event bus
/// * the shutdown signal receiver
fn add_node_resources(builder: EntryNodeBuilder) -> Result<EntryNodeBuilder, EntryNodeError> {
    let node_info = util::create_node_info();

    let config = builder.config().clone();

    let builder = builder
        .with_resource(config)
        .with_resource(node_info)
        .with_resource(Bus::<TypeId>::default());

    #[cfg(unix)]
    let shutdown_rx = shutdown::shutdown_listener(vec![SignalKind::interrupt(), SignalKind::terminate()]);

    #[cfg(not(unix))]
    let shutdown_rx = shutdown::shutdown_listener();

    let builder = builder.with_resource(shutdown_rx);

    Ok(builder)
}

/// Initializes the (optional) autopeering service.
async fn initialize_autopeering(
    builder: EntryNodeBuilder,
) -> Result<(bee_autopeering::event::EventRx, EntryNodeBuilder), EntryNodeError> {
    log::info!("Initializing autopeering...");

    let network_name = builder.config().network_spec().name().to_string();

    // The neighbor validator that includes/excludes certain peers by applying custom criteria.
    let neighbor_validator = EntryNodeNeighborValidator::new();

    // The peer store for persisting discovered peers.
    let peerstore_cfg = SledPeerStoreConfig::new().path(PEERSTORE_PATH);

    // A local entity that can sign outgoing messages, and announce services.
    let keypair = builder.config().local().keypair().clone();
    let local = create_local_autopeering_entity(keypair, builder.config());

    let quit_signal = tokio::signal::ctrl_c();

    let autopeering_rx = bee_autopeering::init::<SledPeerStore, _, _, _>(
        builder.config().autopeering.clone(),
        AUTOPEERING_VERSION,
        network_name,
        local,
        peerstore_cfg,
        quit_signal,
        neighbor_validator,
    )
    .await
    .map_err(|e| EntryNodeError::AutopeeringInitialization(e))?;

    Ok((autopeering_rx, builder))
}

/// Creates the local entity from a ED25519 keypair and a set of provided services.
fn create_local_autopeering_entity(keypair: Keypair, config: &EntryNodeConfig) -> bee_autopeering::Local {
    let local = bee_autopeering::Local::from_keypair(keypair);

    let mut write = local.write();

    // Announce the autopeering service.
    write.add_service(
        AUTOPEERING_SERVICE_NAME,
        ServiceProtocol::Udp,
        config.autopeering.bind_addr.port(),
    );

    drop(write);

    local
}

#[derive(Clone)]
struct EntryNodeNeighborValidator {}

impl EntryNodeNeighborValidator {
    pub fn new() -> Self {
        Self {}
    }
}

impl NeighborValidator for EntryNodeNeighborValidator {
    fn is_valid(&self, _peer: &bee_autopeering::Peer) -> bool {
        // Deny any peering attempt.
        false
    }
}

#[derive(Default)]
pub(crate) struct AutopeeringEventLogger {}

#[async_trait]
impl<N: Node> Worker<N> for AutopeeringEventLogger {
    type Config = bee_autopeering::event::EventRx;
    type Error = Infallible;

    async fn start(node: &mut N, event_rx: Self::Config) -> Result<Self, Self::Error> {
        node.spawn::<Self, _, _>(|shutdown_rx| async move {
            log::info!("Running.");

            let mut event_rx = ShutdownStream::new(shutdown_rx, UnboundedReceiverStream::new(event_rx));

            while let Some(e) = event_rx.next().await {
                log::info!("{}", e);
            }

            log::info!("Stopped.");
        });

        Ok(Self::default())
    }
}
