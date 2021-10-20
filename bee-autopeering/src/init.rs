// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    delay::{DelayBuilder, DelayMode, Repeat as _},
    discovery::{DiscoveryEventRx, DiscoveryManager, DiscoveryManagerConfig},
    local::Local,
    packet::{IncomingPacket, OutgoingPacket},
    peering::{PeeringEventRx, PeeringManager, PeeringManagerConfig},
    peerstore::{InMemoryPeerStore, PeerStore},
    request::RequestManager,
    server::{server_chan, IncomingPacketSenders, Server, ServerConfig, ServerSocket},
    service_map::ServiceMap,
    shutdown::ShutdownBus,
    time,
};

use std::{error, future::Future, ops::DerefMut as _};

/// Initializes the autopeering service.
pub async fn init<S, Q>(
    config: AutopeeringConfig,
    version: u32,
    network_id: u32,
    local: Local,
    peerstore_config: <S as PeerStore>::Config,
    quit_signal: Q,
) -> Result<(DiscoveryEventRx, PeeringEventRx), Box<dyn error::Error>>
where
    S: PeerStore + 'static,
    Q: Future + Send + 'static,
{
    // Create a bus to distribute the shutdown signal to all spawned tasks.
    let (shutdown_bus, mut shutdown_reg) = ShutdownBus::<4>::new();
    tokio::spawn(async move {
        quit_signal.await;
        shutdown_bus.trigger();
    });

    // Create or load a peer store.
    let peerstore = S::new(peerstore_config);

    // Create channels for inbound/outbound communication with the UDP socket.
    let (discovery_tx, discovery_rx) = server_chan::<IncomingPacket>();
    let (peering_tx, peering_rx) = server_chan::<IncomingPacket>();

    let incoming_senders = IncomingPacketSenders {
        discover_tx: discovery_tx,
        peering_tx,
    };

    // Spawn the server handling the socket I/O.
    let server_config = ServerConfig::new(&config);
    // Unwrap: we ensured there are enough items in the vec.
    let (server, outgoing_tx) = Server::new(
        server_config,
        local.clone(),
        incoming_senders,
        shutdown_reg.register(),
        shutdown_reg.register(),
    );

    tokio::spawn(server.run());

    // Create a request manager that creates and keeps track of outgoing requests.
    let request_mngr = RequestManager::new(version, network_id, config.bind_addr, local.clone());

    // Spawn a cronjob that regularly removes unanswered pings.
    let delay = DelayBuilder::new(DelayMode::Constant(1000)).finish();
    let cmd = Box::new(|mngr: &RequestManager| {
        let now = time::unix_now_secs();
        let mut guard = mngr.open_requests.write().expect("error getting write access");
        let requests = guard.deref_mut();
        requests.retain(|_, v| v.expiration_time > now);
    });

    // tokio::spawn(RequestManager::repeat(delay, cmd, request_mngr.clone()));

    // Spawn the discovery manager handling discovery requests/responses.
    let discovery_config = DiscoveryManagerConfig::new(&config, version, network_id);
    let discovery_socket = ServerSocket::new(discovery_rx, outgoing_tx.clone());
    let (discovery_mngr, discovery_event_rx) = DiscoveryManager::new(
        discovery_config,
        local.clone(),
        discovery_socket,
        request_mngr.clone(),
        peerstore.clone(),
        shutdown_reg.register(),
    );

    tokio::spawn(discovery_mngr.run());

    // Spawn the autopeering manager handling peering requests/responses/drops and the storage I/O.
    let peering_config = PeeringManagerConfig::new(&config, version, network_id);
    let peering_socket = ServerSocket::new(peering_rx, outgoing_tx);
    let (peering_mngr, peering_event_rx) = PeeringManager::new(
        peering_config,
        local.clone(),
        peering_socket,
        request_mngr,
        peerstore,
        shutdown_reg.register(),
    );

    tokio::spawn(peering_mngr.run());

    log::debug!("Autopeering initialized.");

    Ok((discovery_event_rx, peering_event_rx))
}
