// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    packets::{IncomingPacket, OutgoingPacket},
    peer::DiscoveredPeer,
};

use tokio::sync::mpsc;

type PacketTx = mpsc::UnboundedSender<OutgoingPacket>;
type PacketRx = mpsc::UnboundedReceiver<IncomingPacket>;

pub(crate) struct AutopeeringManager {
    // Channel half for receiving autopeering related packets.
    receiver: PacketRx,
    // Channel half for sending autopeering related packets.
    sender: PacketTx,
    // Storage for discovered peers
    store: (),
    // Config
    config: AutopeeringConfig,
}

impl AutopeeringManager {
    pub(crate) fn new(rx: PacketRx, tx: PacketTx, config: AutopeeringConfig) -> Self {
        // TODO: read the store
        let store = ();

        Self {
            receiver: rx,
            sender: tx,
            store,
            config,
        }
    }

    pub(crate) async fn run(self) {
        let AutopeeringManager {
            receiver: rx,
            sender: tx,
            store,
            config,
        } = self;

        // contact the entry nodes
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
