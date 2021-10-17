// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use tokio::sync::mpsc;

use crate::{
    config::AutopeeringConfig,
    identity::LocalId,
    packet::{MessageType, OutgoingPacket},
    peering_messages::PeeringRequest,
    request::RequestManager,
    salt::Salt,
    server::ServerSocket,
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

/// Peering related events.
#[derive(Debug)]
pub enum PeeringEvent {
    // hive.go: A SaltUpdated event is triggered, when the private and public salt were updated.
    SaltUpdated,
    // hive.go: An OutgoingPeering event is triggered, when a valid response of PeeringRequest has been received.
    OutgoingPeering,
    // hive.go: An IncomingPeering event is triggered, when a valid PeerRequest has been received.
    IncomingPeering,
    // hive.go: A Dropped event is triggered, when a neighbor is dropped or when a drop message is received.
    Dropped,
}

/// Esposes discovery related events.
pub type PeeringEventRx = mpsc::UnboundedReceiver<PeeringEvent>;
type PeeringEventTx = mpsc::UnboundedSender<PeeringEvent>;

fn event_chan() -> (PeeringEventTx, PeeringEventRx) {
    mpsc::unbounded_channel::<PeeringEvent>()
}

pub(crate) struct PeeringManager {
    config: PeeringConfig,
    // The local id to sign outgoing packets.
    local_id: LocalId,
    // Channel halfs for sending/receiving peering related packets.
    socket: ServerSocket,
    // Handles requests.
    request_mngr: RequestManager,
    // Storage for discovered peers
    store: (),
    // Publishes peering related events.
    event_tx: PeeringEventTx,
}

impl PeeringManager {
    pub(crate) fn new(
        config: PeeringConfig,
        local_id: LocalId,
        socket: ServerSocket,
        request_mngr: RequestManager,
    ) -> (Self, PeeringEventRx) {
        let (event_tx, event_rx) = event_chan();
        (
            Self {
                config,
                local_id,
                socket,
                request_mngr,
                store: (),
                event_tx,
            },
            event_rx,
        )
    }

    pub(crate) async fn run(self) {
        let PeeringManager {
            config,
            local_id,
            socket,
            request_mngr,
            store,
            event_tx,
        } = self;

        // Create a peering request
        let peering_req = request_mngr.new_peering_request();
        let msg_bytes = peering_req.protobuf().expect("error encoding peering request").to_vec();

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
