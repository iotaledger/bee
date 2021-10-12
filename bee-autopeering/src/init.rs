// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    discovery::DiscoveryManager,
    packets::{IncomingPacket, OutgoingPacket},
    peering::PeeringManager,
    server::Server,
};

use tokio::sync::mpsc;

use std::error;

/// Initializes the autopeering service.
pub async fn init(config: AutopeeringConfig) -> Result<(), Box<dyn error::Error>> {
    // Create 2 channels for inbound/outbound communication with the UDP socket.
    let (incoming_send, incoming_recv) = mpsc::unbounded_channel::<IncomingPacket>();
    let (outgoing_send, outgoing_recv) = mpsc::unbounded_channel::<OutgoingPacket>();

    // Spawn the server handling the socket I/O.
    let srvr = Server::new(incoming_send, outgoing_recv, config.clone());
    tokio::spawn(srvr.run());

    // Spawn the discovery manager handling discovery requests/responses.
    let discovery_mngr = DiscoveryManager::new();
    tokio::spawn(discovery_mngr.run());

    // Spawn the autopeering manager handling peering requests/responses/drops and the storage I/O.
    let peering_mngr = PeeringManager::new(incoming_recv, outgoing_send, config);
    tokio::spawn(peering_mngr.run());

    Ok(())
}
