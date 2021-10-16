// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    discovery::{DiscoveryConfig, DiscoveryEventRx, DiscoveryManager},
    identity::LocalId,
    packet::{IncomingPacket, OutgoingPacket},
    peering::{PeeringConfig, PeeringEventRx, PeeringManager},
    server::{server_chan, IncomingPacketSenders, Server, ServerConfig, ServerSocket},
    service_map::ServiceMap,
};

use std::error;

/// Initializes the autopeering service.
pub async fn init(
    config: AutopeeringConfig,
    version: u32,
    network_id: u32,
    local_id: LocalId,
    services: ServiceMap,
) -> Result<(DiscoveryEventRx, PeeringEventRx), Box<dyn error::Error>> {
    // Create channels for inbound/outbound communication with the UDP socket.
    let (discovery_tx, discovery_rx) = server_chan::<IncomingPacket>();
    let (peering_tx, peering_rx) = server_chan::<IncomingPacket>();

    let incoming_senders = IncomingPacketSenders {
        discovery_tx,
        peering_tx,
    };

    // Spawn the server handling the socket I/O.
    let server_config = ServerConfig::new(&config);
    let (server, outgoing_tx) = Server::new(server_config, local_id.clone(), incoming_senders);

    tokio::spawn(server.run());

    // Spawn the discovery manager handling discovery requests/responses.
    let discovery_config = DiscoveryConfig::new(&config, version, network_id, services);
    let discovery_socket = ServerSocket::new(discovery_rx, outgoing_tx.clone());
    let (discovery_mngr, discovery_event_rx) =
        DiscoveryManager::new(discovery_config, local_id.clone(), discovery_socket);

    tokio::spawn(discovery_mngr.run());

    // Spawn the autopeering manager handling peering requests/responses/drops and the storage I/O.
    let peering_config = PeeringConfig::new(&config, version, network_id);
    let peering_socket = ServerSocket::new(peering_rx, outgoing_tx);
    let (peering_mngr, peering_event_rx) = PeeringManager::new(peering_config, local_id.clone(), peering_socket);

    tokio::spawn(peering_mngr.run());

    Ok((discovery_event_rx, peering_event_rx))
}
