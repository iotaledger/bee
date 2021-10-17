// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    delay::{DelayBuilder, DelayMode, Repeat as _},
    discovery::{DiscoveryEventRx, DiscoveryManager, DiscoveryManagerConfig},
    identity::LocalId,
    packet::{IncomingPacket, OutgoingPacket},
    peering::{PeeringConfig, PeeringEventRx, PeeringManager},
    request::RequestManager,
    server::{server_chan, IncomingPacketSenders, Server, ServerConfig, ServerSocket},
    service_map::ServiceMap,
    store::InMemoryPeerStore,
    time,
};

use std::{error, ops::DerefMut as _};

/// Initializes the autopeering service.
pub async fn init(
    config: AutopeeringConfig,
    version: u32,
    network_id: u32,
    local_id: LocalId,
    services: ServiceMap,
) -> Result<(DiscoveryEventRx, PeeringEventRx), Box<dyn error::Error>> {
    // Create channels for inbound/outbound communication with the UDP socket.
    let (discover_tx, discover_rx) = server_chan::<IncomingPacket>();
    let (peering_tx, peering_rx) = server_chan::<IncomingPacket>();

    let incoming_senders = IncomingPacketSenders {
        discover_tx,
        peering_tx,
    };

    // Spawn the server handling the socket I/O.
    let server_config = ServerConfig::new(&config);
    let (server, outgoing_tx) = Server::new(server_config, local_id.clone(), incoming_senders);

    tokio::spawn(server.run());

    // Create a request manager that creates and keeps track of outgoing requests.
    let request_mngr = RequestManager::new(version, network_id, config.bind_addr, local_id.clone());

    // Spawn a cronjob that regularly removes unanswered pings.
    let delay = DelayBuilder::new(DelayMode::Constant(1000)).finish();
    let cmd = Box::new(|mngr: &RequestManager| {
        let now = time::unix_now();
        let mut guard = mngr.open_requests.write().expect("error getting write access");
        let requests = guard.deref_mut();
        requests.retain(|_, v| v.expiration_time > now);
    });

    tokio::spawn(RequestManager::repeat(delay, cmd, request_mngr.clone()));

    // Spawn the discovery manager handling discovery requests/responses.
    let discover_config = DiscoveryManagerConfig::new(&config, version, network_id, services);
    let discover_socket = ServerSocket::new(discover_rx, outgoing_tx.clone());
    let (discover_mngr, discover_event_rx) =
        DiscoveryManager::new(discover_config, local_id.clone(), discover_socket, request_mngr.clone());

    tokio::spawn(discover_mngr.run());

    // Create a peer store
    let peer_store = InMemoryPeerStore::default();

    // Spawn the autopeering manager handling peering requests/responses/drops and the storage I/O.
    let peering_config = PeeringConfig::new(&config, version, network_id);
    let peering_socket = ServerSocket::new(peering_rx, outgoing_tx);
    let (peering_mngr, peering_event_rx) = PeeringManager::new(
        peering_config,
        local_id.clone(),
        peering_socket,
        request_mngr,
        peer_store,
    );

    tokio::spawn(peering_mngr.run());

    Ok((discover_event_rx, peering_event_rx))
}
