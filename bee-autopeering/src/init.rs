// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    manager::AutopeeringManager,
    packets::{IncomingPacket, OutgoingPacket},
    server::AutopeeringServer,
};

use tokio::sync::mpsc;

use std::error;

/// Initializes the autopeering service.
pub async fn init(config: AutopeeringConfig) -> Result<(), Box<dyn error::Error>> {
    // Create 2 channels for inbound/outbound communication with the UDP socket.
    let (incoming_send, incoming_recv) = mpsc::unbounded_channel::<IncomingPacket>();
    let (outgoing_send, outgoing_recv) = mpsc::unbounded_channel::<OutgoingPacket>();

    // Spawn the autopeering server handling the socket I/O.
    let srvr = AutopeeringServer::new(incoming_send, outgoing_recv, config.clone());
    tokio::spawn(srvr.run());

    // Spawn the autopeering manager handling the peering requests/responses/drops and the storage I/O.
    let mngr = AutopeeringManager::new(incoming_recv, outgoing_send, config);
    tokio::spawn(mngr.run());

    Ok(())
}
