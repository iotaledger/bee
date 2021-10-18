// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use tokio::sync::mpsc;

use crate::{
    config::AutopeeringConfig,
    hash,
    local::Local,
    packet::{IncomingPacket, MessageType, OutgoingPacket},
    peering_messages::{PeeringDrop, PeeringRequest, PeeringResponse},
    peerstore::{InMemoryPeerStore, PeerStore},
    request::RequestManager,
    salt::Salt,
    server::ServerSocket,
    PeerId,
};

use std::{net::SocketAddr, time::Duration};

pub(crate) struct PeeringConfig {
    pub(crate) version: u32,
    pub(crate) network_id: u32,
    pub(crate) source_addr: SocketAddr,
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

pub(crate) struct PeeringManager<S> {
    config: PeeringConfig,
    // The local peer.
    local: Local,
    // Channel halfs for sending/receiving peering related packets.
    socket: ServerSocket,
    // Handles requests.
    request_mngr: RequestManager,
    // Publishes peering related events.
    event_tx: PeeringEventTx,
    // The storage for discovered peers.
    peerstore: S,
}

impl<S: PeerStore> PeeringManager<S> {
    pub(crate) fn new(
        config: PeeringConfig,
        local: Local,
        socket: ServerSocket,
        request_mngr: RequestManager,
        peerstore: S,
    ) -> (Self, PeeringEventRx) {
        let (event_tx, event_rx) = event_chan();
        (
            Self {
                config,
                local,
                socket,
                request_mngr,
                event_tx,
                peerstore,
            },
            event_rx,
        )
    }

    pub(crate) async fn run(self) {
        let PeeringManager {
            config,
            local,
            socket,
            request_mngr,
            event_tx,
            peerstore,
        } = self;

        let PeeringConfig {
            version,
            network_id,
            source_addr,
        } = config;

        let ServerSocket { mut rx, tx } = socket;

        loop {
            if let Some(IncomingPacket {
                msg_type,
                msg_bytes,
                source_addr,
                peer_id,
            }) = rx.recv().await
            {
                match msg_type {
                    MessageType::PeeringRequest => {
                        let peer_req =
                            PeeringRequest::from_protobuf(&msg_bytes).expect("error decoding peering request");

                        if !validate_peering_request(&peer_req) {
                            log::debug!("Received invalid peering request: {:?}", peer_req);
                            continue;
                        }

                        let request_hash = &hash::sha256(&msg_bytes)[..];

                        send_peering_response(request_hash, &tx, source_addr);
                    }
                    MessageType::PeeringResponse => {
                        let peer_res =
                            PeeringResponse::from_protobuf(&msg_bytes).expect("error decoding peering response");

                        if !validate_peering_response(&peer_res, &request_mngr, &peer_id) {
                            log::debug!("Received invalid peering response: {:?}", peer_res);
                            continue;
                        }

                        handle_peering_response();
                    }
                    MessageType::PeeringDrop => {
                        let peer_drop =
                            PeeringDrop::from_protobuf(&msg_bytes).expect("error decoding discover request");

                        if !validate_peering_drop(&peer_drop) {
                            log::debug!("Received invalid peering drop: {:?}", peer_drop);
                            continue;
                        }

                        handle_peering_drop();
                    }
                    _ => panic!("unsupported peering message type"),
                }
            }
        }
    }
}

fn validate_peering_request(peer_req: &PeeringRequest) -> bool {
    todo!()
}

fn send_peering_response(request_hash: &[u8], tx: &mpsc::UnboundedSender<OutgoingPacket>, source_addr: SocketAddr) {
    todo!()
}

fn validate_peering_response(peer_res: &PeeringResponse, request_mngr: &RequestManager, peer_id: &PeerId) -> bool {
    todo!()
}

fn handle_peering_response() {
    todo!()
}

fn validate_peering_drop(peer_drop: &PeeringDrop) -> bool {
    todo!()
}

fn handle_peering_drop() {
    todo!()
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
