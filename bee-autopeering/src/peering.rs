// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    identity::LocalId,
    packet::{MessageType, OutgoingPacket, Socket},
    peering_messages::PeeringRequest,
    salt::Salt,
};

use std::{net::SocketAddr, time::Duration};

pub(crate) struct PeeringConfig {
    pub version: u32,
    pub network_id: u32,
    pub source_addr: SocketAddr,
}

impl PeeringConfig {
    pub fn new(config: &AutopeeringConfig, version: u32, network_id: u32) -> Self {
        Self {
            version,
            network_id,
            source_addr: config.bind_addr,
        }
    }
}

pub(crate) struct PeeringManager {
    config: PeeringConfig,
    // The local id to sign outgoing packets.
    local_id: LocalId,
    // Channel halfs for sending/receiving peering related packets.
    socket: Socket,
    // Storage for discovered peers
    store: (),
}

impl PeeringManager {
    pub(crate) fn new(config: PeeringConfig, local_id: LocalId, socket: Socket) -> Self {
        Self {
            config,
            local_id,
            socket,
            store: (),
        }
    }

    pub(crate) async fn run(self) {
        let PeeringManager {
            config,
            local_id,
            socket,
            store,
        } = self;

        let salt = Salt::new(Duration::from_secs(20));

        // Create a peering request
        let msg_bytes = PeeringRequest::new(salt.bytes().to_vec(), salt.expiration_time())
            .protobuf()
            .expect("error encoding peering request")
            .to_vec();

        let packet = OutgoingPacket {
            msg_type: MessageType::PeeringRequest,
            msg_bytes,
            // FIXME
            target_addr: "127.0.0.1:1337".parse().expect("FIXME"),
        };

        socket.send(packet);
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("response timeout")]
    ResponseTimeout,
    #[error("socket was closed")]
    SocketClosed,
    #[error("packet does not contain a message")]
    NoMessage,
    #[error("packet contains an invalid message")]
    InvalidMessage,
}
