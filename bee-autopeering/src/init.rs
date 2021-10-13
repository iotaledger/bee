// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    discovery::{DiscoveryConfig, DiscoveryManager},
    packets::{IncomingPacket, OutgoingPacket},
    peering::PeeringManager,
    server::{Server, ServerConfig},
};

use tokio::sync::mpsc;

use std::error;

/// Initializes the autopeering service.
pub async fn init(config: AutopeeringConfig, version: u32, network_id: u32) -> Result<(), Box<dyn error::Error>> {
    let server_config = ServerConfig::new(&config);
    let discovery_config = DiscoveryConfig::new(&config, version, network_id);

    // Create 2 channels for inbound/outbound communication with the UDP socket.
    let (incoming_send, incoming_recv) = mpsc::unbounded_channel::<IncomingPacket>();
    let (outgoing_send, outgoing_recv) = mpsc::unbounded_channel::<OutgoingPacket>();

    // Spawn the server handling the socket I/O.
    let srvr = Server::new(server_config, incoming_send, outgoing_recv);
    tokio::spawn(srvr.run());

    // Spawn the discovery manager handling discovery requests/responses.
    let discovery_mngr = DiscoveryManager::new(discovery_config, incoming_recv, outgoing_send);
    tokio::spawn(discovery_mngr.run());

    // // Spawn the autopeering manager handling peering requests/responses/drops and the storage I/O.
    // let peering_mngr = PeeringManager::new(incoming_recv, outgoing_send, config);
    // tokio::spawn(peering_mngr.run());

    Ok(())
}
