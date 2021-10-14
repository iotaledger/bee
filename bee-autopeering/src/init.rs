// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    discovery::{DiscoveryConfig, DiscoveryManager},
    identity::LocalId,
    packet::{IncomingPacket, OutgoingPacket, Socket},
    peering::{PeeringConfig, PeeringManager},
    server::{PacketTxs, Server, ServerConfig},
};

use tokio::sync::mpsc::unbounded_channel as chan;

use std::error;

/// Initializes the autopeering service.
pub async fn init(
    config: AutopeeringConfig,
    version: u32,
    network_id: u32,
    local_id: LocalId,
) -> Result<(), Box<dyn error::Error>> {
    // Create 2 channels for inbound/outbound communication with the UDP socket.
    let (discovery_tx, discovery_rx) = chan::<IncomingPacket>();
    let (peering_tx, peering_rx) = chan::<IncomingPacket>();
    let (outgoing_tx, outgoing_rx) = chan::<OutgoingPacket>();

    let incoming_txs = PacketTxs {
        discovery_tx,
        peering_tx,
    };

    // Spawn the server handling the socket I/O.
    let server_config = ServerConfig::new(&config);
    let server = Server::new(server_config, incoming_txs, outgoing_rx);
    tokio::spawn(server.run());

    // Spawn the discovery manager handling discovery requests/responses.
    let discovery_config = DiscoveryConfig::new(&config, version, network_id);
    let discovery_socket = Socket::new(discovery_rx, outgoing_tx.clone());
    let discovery_mngr = DiscoveryManager::new(discovery_config, local_id.clone(), discovery_socket);
    tokio::spawn(discovery_mngr.run());

    // Spawn the autopeering manager handling peering requests/responses/drops and the storage I/O.
    let peering_config = PeeringConfig::new(&config, version, network_id);
    let peering_socket = Socket::new(peering_rx, outgoing_tx);
    let peering_mngr = PeeringManager::new(peering_config, local_id.clone(), peering_socket);
    tokio::spawn(peering_mngr.run());

    Ok(())
}
