// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Autopeering initialization.

use crate::{
    config::AutopeeringConfig,
    delay,
    discovery::{
        manager::{DiscoveryManager, DiscoveryManagerConfig, QUERY_INTERVAL_DEFAULT, REVERIFY_INTERVAL_DEFAULT},
        query::{self, QueryContext},
    },
    event::{self, EventRx},
    hash,
    local::Local,
    multiaddr,
    packet::IncomingPacket,
    peer::{
        lists::{ActivePeersList, EntryPeersList, ReplacementPeersList},
        stores::PeerStore,
    },
    peering::{
        filter::NeighborFilter,
        manager::{InboundNeighborhood, OutboundNeighborhood, PeeringManager, SaltUpdateContext, SALT_UPDATE_SECS},
        update::{self, UpdateContext, OPEN_OUTBOUND_NBH_UPDATE_SECS},
        NeighborValidator,
    },
    request::{self, RequestManager, EXPIRED_REQUEST_REMOVAL_INTERVAL},
    server::{server_chan, IncomingPacketSenders, Server, ServerConfig, ServerSocket},
    task::{TaskManager, MAX_SHUTDOWN_PRIORITY},
    time::SECOND,
};

use std::{error, future::Future, iter, time::Duration};

const NUM_TASKS: usize = 9;
const BOOTSTRAP_MAX_VERIFICATIONS: usize = 10;
const BOOTSTRAP_VERIFICATION_DELAY: Duration = Duration::from_millis(100);
const BOOTSTRAP_QUERY_DELAY: Duration = Duration::from_secs(2 * SECOND);
const BOOTSTRAP_UPDATE_DELAY: Duration = Duration::from_secs(4 * SECOND);

/// Initializes the autopeering service.
pub async fn init<S, I, Q, V>(
    config: AutopeeringConfig,
    version: u32,
    network_name: I,
    local: Local,
    peer_store_config: <S as PeerStore>::Config,
    term_signal: Q,
    neighbor_validator: V,
) -> Result<EventRx, Box<dyn error::Error>>
where
    S: PeerStore + 'static,
    I: AsRef<str>,
    Q: Future + Send + 'static,
    V: NeighborValidator + 'static,
{
    let network_id = hash::network_hash(&network_name);

    log::info!("---------------------------------------------------------------------------------------------------");
    log::info!("WARNING:");
    log::info!("Autopeering will disclose your public IP address to possibly all nodes and entry points.");
    log::info!("Please disable it if you do not want this to happen!");
    log::info!("---------------------------------------------------------------------------------------------------");
    log::info!("Network name/id: {}/{}", network_name.as_ref(), network_id);
    log::info!("Protocol_version: {}", version);
    log::info!("Public key: {}", multiaddr::pubkey_to_base58(&local.public_key()));
    log::info!("Bind address: {}", config.bind_addr());

    // Create or load a peer store.
    let peer_store = S::new(peer_store_config)?;

    // Create peer lists.
    let entry_peers = EntryPeersList::default();
    let active_peers = ActivePeersList::default();
    let replacements = ReplacementPeersList::default();

    // Create a task manager to have good control over the tokio task spawning business.
    let mut task_mngr =
        TaskManager::<_, NUM_TASKS>::new(peer_store.clone(), active_peers.clone(), replacements.clone());

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
    let request_mngr = RequestManager::new(version, network_id, config.bind_addr());

    // Create the discovery manager handling the discovery request/response protocol.
    let discovery_config = DiscoveryManagerConfig::new(&config, version, network_id);
    let discovery_socket = ServerSocket::new(discovery_rx, server_tx.clone());

    let discovery_mngr = DiscoveryManager::new(
        discovery_config,
        local.clone(),
        discovery_socket,
        request_mngr.clone(),
        peer_store.clone(),
        entry_peers.clone(),
        active_peers.clone(),
        replacements.clone(),
        event_tx.clone(),
    );
    discovery_mngr.init(&mut task_mngr).await?;

    // Create neighborhoods and neighbor candidate filter.
    let inbound_nbh = InboundNeighborhood::new();
    let outbound_nbh = OutboundNeighborhood::new();
    let nb_filter = NeighborFilter::new(local.peer_id(), neighbor_validator);

    // Create the autopeering manager handling the peering request/response protocol.
    let peering_socket = ServerSocket::new(peering_rx, server_tx.clone());

    let peering_mngr = PeeringManager::new(
        local.clone(),
        peering_socket,
        request_mngr.clone(),
        active_peers.clone(),
        event_tx.clone(),
        inbound_nbh.clone(),
        outbound_nbh.clone(),
        nb_filter.clone(),
    );
    task_mngr.run(peering_mngr);

    // TODO: remove this when sure that all open requests are garbage collected.
    // Remove expired requests regularly.
    let f = request::remove_expired_requests_fn();
    let delay = iter::repeat(EXPIRED_REQUEST_REMOVAL_INTERVAL);
    let ctx = request_mngr.clone();
    task_mngr.repeat(f, delay, ctx, "Expired-Request-Removal", MAX_SHUTDOWN_PRIORITY);

    let ctx = SaltUpdateContext::new(
        local.clone(),
        nb_filter.clone(),
        inbound_nbh,
        outbound_nbh.clone(),
        server_tx.clone(),
        event_tx.clone(),
    );

    // Update salts regularly.
    let f = crate::peering::manager::update_salts_fn(config.drop_neighbors_on_salt_update());
    let delay = iter::repeat(SALT_UPDATE_SECS);
    task_mngr.repeat(f, delay, ctx, "Salt-Update", MAX_SHUTDOWN_PRIORITY);

    let ctx = QueryContext {
        request_mngr: request_mngr.clone(),
        entry_peers: entry_peers.clone(),
        active_peers: active_peers.clone(),
        replacements: replacements.clone(),
        server_tx: server_tx.clone(),
        event_tx: event_tx.clone(),
    };

    // Reverify old peers regularly.
    let f = query::reverify_fn();
    let delay = iter::repeat(BOOTSTRAP_VERIFICATION_DELAY)
        .take(BOOTSTRAP_MAX_VERIFICATIONS.min(active_peers.read().len()))
        .chain(iter::repeat(REVERIFY_INTERVAL_DEFAULT));
    task_mngr.repeat(f, delay, ctx.clone(), "Reverification", MAX_SHUTDOWN_PRIORITY);

    // Discover new peers regularly.
    let f = query::query_fn();
    let delay = iter::once(BOOTSTRAP_QUERY_DELAY).chain(iter::repeat(QUERY_INTERVAL_DEFAULT));
    task_mngr.repeat(f, delay, ctx, "Discovery", MAX_SHUTDOWN_PRIORITY);

    let ctx = UpdateContext {
        local,
        request_mngr,
        active_peers,
        nb_filter,
        outbound_nbh,
        server_tx,
    };

    // Update the outbound neighborhood regularly (interval depends on whether slots available or not).
    let f = update::update_outbound_neighborhood_fn();
    let delay = iter::once(BOOTSTRAP_UPDATE_DELAY).chain(delay::ManualDelayFactory::new(OPEN_OUTBOUND_NBH_UPDATE_SECS));
    task_mngr.repeat(f, delay, ctx, "Outbound neighborhood update", MAX_SHUTDOWN_PRIORITY);

    // Await the shutdown signal (in a separate task).
    tokio::spawn(async move {
        term_signal.await;
        task_mngr.shutdown().await?;
        Ok::<_, S::Error>(())
    });

    log::debug!("Autopeering initialized.");

    Ok(event_rx)
}
