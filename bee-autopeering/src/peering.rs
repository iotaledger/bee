// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    messages::PeeringRequest,
    packets::{IncomingPacket, OutgoingPacket},
    salt::Salt,
};

use tokio::sync::mpsc;

use std::time::Duration;

type PacketTx = mpsc::UnboundedSender<OutgoingPacket>;
type PacketRx = mpsc::UnboundedReceiver<IncomingPacket>;

pub(crate) struct PeeringManager {
    // Channel half for receiving autopeering related packets.
    rx: PacketRx,
    // Channel half for sending autopeering related packets.
    tx: PacketTx,
    // Storage for discovered peers
    store: (),
    // Config
    config: AutopeeringConfig,
}

impl PeeringManager {
    pub(crate) fn new(rx: PacketRx, tx: PacketTx, config: AutopeeringConfig) -> Self {
        // TODO: read the store
        let store = ();

        Self { rx, tx, store, config }
    }

    pub(crate) async fn run(self) {
        let PeeringManager { rx, tx, store, config } = self;

        let salt = Salt::new(Duration::from_secs(20));

        let request_bytes = PeeringRequest::new(salt.bytes().to_vec(), salt.expiration_time())
            .protobuf()
            .expect("error creating peering request");

        // contact the entry nodes
        for multiaddr in config.entry_nodes {
            let packet = OutgoingPacket {
                bytes: request_bytes.to_vec(),
                target: multiaddr.host_socketaddr(),
            };

            tx.send(packet);
        }
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
