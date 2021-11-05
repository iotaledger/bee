// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Autopeering initialization.

use crate::{
    config::AutopeeringConfig,
    delay,
    discovery::{
        manager::{
            DiscoveryManager, DiscoveryManagerConfig, DEFAULT_QUERY_INTERVAL_SECS, DEFAULT_REVERIFY_INTERVAL_SECS,
        },
        query::{self, QueryContext},
    },
    event::{self, EventRx},
    hash,
    local::Local,
    multiaddr,
    packet::IncomingPacket,
    peer::{
        peerlist::{ActivePeersList, MasterPeersList, ReplacementList},
        peerstore::PeerStore,
    },
    peering::{
        filter::NeighborFilter,
        manager::{
            InboundNeighborhood, OutboundNeighborhood, PeeringManager, PeeringManagerConfig, SaltUpdateContext,
            SALT_UPDATE_SECS,
        },
        update::{self, UpdateContext, OPEN_OUTBOUND_NBH_UPDATE_SECS},
        NeighborValidator,
    },
    request::{self, RequestManager, EXPIRED_REQUEST_REMOVAL_INTERVAL_CHECK_SECS},
    server::{server_chan, IncomingPacketSenders, Server, ServerConfig, ServerSocket},
    task::{TaskManager, MAX_SHUTDOWN_PRIORITY},
    time,
};

use std::{error, future::Future, iter, time::Duration};

const NUM_TASKS: usize = 9;

/// Initializes the autopeering service.
pub async fn init<S, I, Q, V>(
    config: AutopeeringConfig,
    version: u32,
    network_name: I,
    local: Local,
    peerstore_config: <S as PeerStore>::Config,
    quit_signal: Q,
    neighbor_validator: V,
) -> Result<EventRx, Box<dyn error::Error>>
where
    S: PeerStore + 'static,
    I: AsRef<str>,
    Q: Future + Send + 'static,
    V: NeighborValidator + 'static,
{
    let network_id = hash::fnv32(&network_name);
    let private_salt = time::datetime(
        local
            .read()
            .private_salt()
            .expect("missing private salt")
            .expiration_time(),
    );
    let public_salt = time::datetime(
        local
            .read()
            .public_salt()
            .expect("missing private salt")
            .expiration_time(),
    );

    log::info!("---------------------------------------------------------------------------------------------------");
    log::info!("WARNING:");
    log::info!("The autopeering system will disclose your public IP address to possibly all nodes and entry points.");
    log::info!("Please disable it if you do not want this to happen!");
    log::info!("---------------------------------------------------------------------------------------------------");
    log::info!("Network name/id: {}/{}", network_name.as_ref(), network_id);
    log::info!("Protocol_version: {}", version);
    log::info!(
        "Public key: {}",
        multiaddr::from_pubkey_to_base58(local.read().public_key())
    );
    log::info!("Current time: {}", time::datetime_now());
    log::info!("Private salt: {}", private_salt);
    log::info!("Public salt: {}", public_salt);
    log::info!("Bind address: {}", config.bind_addr);

    // Create or load a peer store.
    let peerstore = S::new(peerstore_config);

    // Create peer lists.
    let master_peers = MasterPeersList::default();
    let active_peers = ActivePeersList::default();
    let replacements = ReplacementList::default();

    // Create a task manager to have good control over the tokio task spawning business.
    let mut task_mngr = TaskManager::<_, NUM_TASKS>::new(peerstore.clone(), active_peers.clone(), replacements.clone());

    // Create channels for inbound/outbound communication with the UDP server.
    let (discovery_tx, discovery_rx) = server_chan::<IncomingPacket>();
    let (peering_tx, peering_rx) = server_chan::<IncomingPacket>();
    let incoming_senders = IncomingPacketSenders {
        discovery_tx,
        peering_tx,
    };

    // Event channel to publish events to the user.
    let (event_tx, event_rx) = event::event_chan();

    // Initialize the server managing the UDP socket I/O.
    let server_config = ServerConfig::new(&config);
    let (server, server_tx) = Server::new(server_config, local.clone(), incoming_senders);
    server.init(&mut task_mngr).await;

    // Create a request manager that creates and keeps track of outgoing requests.
    let request_mngr = RequestManager::new(version, network_id, config.bind_addr);

    // Create the discovery manager handling the discovery request/response protocol.
    let discovery_config = DiscoveryManagerConfig::new(&config, version, network_id);
    let discovery_socket = ServerSocket::new(discovery_rx, server_tx.clone());

    let discovery_mngr = DiscoveryManager::new(
        discovery_config,
        local.clone(),
        discovery_socket,
        request_mngr.clone(),
        peerstore.clone(),
        master_peers.clone(),
        active_peers.clone(),
        replacements.clone(),
        event_tx.clone(),
    );
    let command_tx = discovery_mngr.init(&mut task_mngr).await;

    // Create neighborhoods and neighbor candidate filter.
    let inbound_nbh = InboundNeighborhood::new();
    let outbound_nbh = OutboundNeighborhood::new();
    let nb_filter = NeighborFilter::new(*local.read().peer_id(), neighbor_validator);

    // Create the autopeering manager handling the peering request/response protocol.
    let peering_config = PeeringManagerConfig::new(&config, version, network_id);
    let peering_socket = ServerSocket::new(peering_rx, server_tx.clone());

    let peering_mngr = PeeringManager::new(
        peering_config,
        local.clone(),
        peering_socket,
        request_mngr.clone(),
        active_peers.clone(),
        event_tx.clone(),
        command_tx,
        inbound_nbh.clone(),
        outbound_nbh.clone(),
        nb_filter.clone(),
    );
    task_mngr.run(peering_mngr);

    // Remove expired requests regularly.
    let cmd = request::remove_expired_requests_repeat();
    let delay = iter::repeat(Duration::from_secs(EXPIRED_REQUEST_REMOVAL_INTERVAL_CHECK_SECS));
    let ctx = request_mngr.clone();
    task_mngr.repeat(cmd, delay, ctx, "Expired-Request-Removal", MAX_SHUTDOWN_PRIORITY);

    let ctx = SaltUpdateContext::new(
        local.clone(),
        nb_filter.clone(),
        inbound_nbh.clone(),
        outbound_nbh.clone(),
        server_tx.clone(),
        event_tx.clone(),
    );

    // Update salts regularly.
    let cmd = crate::peering::manager::repeat_update_salts(config.drop_neighbors_on_salt_update);
    let delay = iter::repeat(SALT_UPDATE_SECS);
    task_mngr.repeat(cmd, delay, ctx, "Salt-Update", MAX_SHUTDOWN_PRIORITY);

    let ctx = QueryContext {
        local: local.clone(),
        request_mngr: request_mngr.clone(),
        master_peers: master_peers.clone(),
        active_peers: active_peers.clone(),
        replacements: replacements.clone(),
        server_tx: server_tx.clone(),
        event_tx: event_tx.clone(),
    };

    // Reverify old peers regularly.
    let cmd = query::do_reverify();
    let delay = iter::repeat(Duration::from_secs(DEFAULT_REVERIFY_INTERVAL_SECS));
    task_mngr.repeat(cmd, delay, ctx.clone(), "Reverification", MAX_SHUTDOWN_PRIORITY);

    // Discover new peers regularly.
    let cmd = query::do_query();
    let delay = iter::repeat(Duration::from_secs(DEFAULT_QUERY_INTERVAL_SECS));
    task_mngr.repeat(cmd, delay, ctx, "Discovery", MAX_SHUTDOWN_PRIORITY);

    let ctx = UpdateContext {
        local,
        request_mngr,
        active_peers,
        nb_filter,
        inbound_nbh,
        outbound_nbh,
        server_tx,
        event_tx,
    };

    // Update the outbound neighborhood regularly (interval depends on whether slots available or not).
    let cmd = update::do_update();
    let delay = delay::ManualDelayFactory::new(OPEN_OUTBOUND_NBH_UPDATE_SECS);
    task_mngr.repeat(cmd, delay, ctx, "Outbound neighborhood update", MAX_SHUTDOWN_PRIORITY);

    // Await the shutdown signal (in a separate task).
    tokio::spawn(async move {
        quit_signal.await;
        task_mngr.shutdown().await;
    });

    log::debug!("Autopeering initialized.");

    Ok(event_rx)
}
