// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    discovery,
    distance::{Neighborhood, SIZE_INBOUND, SIZE_OUTBOUND},
    filter::RejectionFilter,
    hash,
    identity::PeerId,
    local::Local,
    packet::{IncomingPacket, MessageType, OutgoingPacket},
    peer::{self, Peer},
    peering_messages::{DropRequest, PeeringRequest, PeeringResponse},
    peerstore::{self, InMemoryPeerStore, PeerStore},
    request::{self, RequestManager},
    salt::{self, Salt},
    server::{ServerSocket, ServerTx},
    service_map::AUTOPEERING_SERVICE_NAME,
    shutdown::ShutdownRx,
};

use tokio::sync::mpsc;

use std::{net::SocketAddr, time::Duration, vec};

const DEFAULT_OUTBOUND_UPDATE_INTERVAL_SECS: u64 = 1;
const DEFAULT_FULL_OUTBOUND_UPDATE_INTERVAL_SECS: u64 = 60;

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

type InboundNeighborhood = Neighborhood<SIZE_INBOUND, true>;
type OutboundNeighborhood = Neighborhood<SIZE_OUTBOUND, false>;

fn event_chan() -> (PeeringEventTx, PeeringEventRx) {
    mpsc::unbounded_channel::<PeeringEvent>()
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

pub(crate) struct PeeringManagerConfig {
    pub(crate) version: u32,
    pub(crate) network_id: u32,
    pub(crate) source_addr: SocketAddr,
    pub(crate) drop_neighbors_on_salt_update: bool,
}

impl PeeringManagerConfig {
    pub fn new(config: &AutopeeringConfig, version: u32, network_id: u32) -> Self {
        Self {
            version,
            network_id,
            source_addr: config.bind_addr,
            drop_neighbors_on_salt_update: false,
        }
    }
}

pub(crate) struct PeeringManager<S> {
    // The peering config.
    config: PeeringManagerConfig,
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
    // Inbound neighborhood.
    inbound_nh: InboundNeighborhood,
    // Outbound neighborhood.
    outbound_nh: OutboundNeighborhood,
    // The peer rejection filter.
    rejection_filter: RejectionFilter,
    // The shutdown signal receiver.
    shutdown_rx: ShutdownRx,
}

impl<S: PeerStore> PeeringManager<S> {
    pub(crate) fn new(
        config: PeeringManagerConfig,
        local: Local,
        socket: ServerSocket,
        request_mngr: RequestManager,
        peerstore: S,
        shutdown_rx: ShutdownRx,
    ) -> (Self, PeeringEventRx) {
        let (event_tx, event_rx) = event_chan();

        let inbound_nh = Neighborhood::new(local.clone());
        let outbound_nh = Neighborhood::new(local.clone());

        let rejection_filter = RejectionFilter::new();

        (
            Self {
                config,
                local,
                socket,
                request_mngr,
                event_tx,
                peerstore,
                inbound_nh,
                outbound_nh,
                rejection_filter,
                shutdown_rx,
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
            mut inbound_nh,
            mut outbound_nh,
            mut rejection_filter,
            mut shutdown_rx,
        } = self;

        let PeeringManagerConfig {
            version,
            network_id,
            source_addr,
            drop_neighbors_on_salt_update,
        } = config;

        let ServerSocket {
            mut server_rx,
            server_tx,
        } = socket;

        loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    break;
                }
                o = server_rx.recv() => {
                    if let Some(IncomingPacket {
                        msg_type,
                        msg_bytes,
                        source_socket_addr,
                        peer_id,
                    }) = o
                    {
                        match msg_type {
                            MessageType::PeeringRequest => {
                                let peering_req =
                                    PeeringRequest::from_protobuf(&msg_bytes).expect("error decoding peering request");

                                if !validate_peering_request(&peering_req, &peer_id, &peerstore) {
                                    log::debug!("Received invalid peering request: {:?}", peering_req);
                                    continue;
                                }

                                handle_peering_request(&peering_req, &msg_bytes, &server_tx, source_addr);
                            }
                            MessageType::PeeringResponse => {
                                let peering_res =
                                    PeeringResponse::from_protobuf(&msg_bytes).expect("error decoding peering response");

                                if !validate_peering_response(&peering_res, &peer_id, &request_mngr) {
                                    log::debug!("Received invalid peering response: {:?}", peering_res);
                                    continue;
                                }

                                handle_peering_response(&peering_res);
                            }
                            MessageType::DropRequest => {
                                let drop_req = DropRequest::from_protobuf(&msg_bytes).expect("error decoding drop request");

                                if !validate_drop_request(&drop_req) {
                                    log::debug!("Received invalid drop request: {:?}", drop_req);
                                    continue;
                                }

                                handle_drop_request(
                                    &drop_req,
                                    peer_id,
                                    &mut inbound_nh,
                                    &mut outbound_nh,
                                    &mut rejection_filter,
                                    &server_tx,
                                    source_addr,
                                );
                            }
                            _ => panic!("unsupported peering message type"),
                        }
                    }
                }
            }
        }
    }
}

fn validate_peering_request<S: PeerStore>(peering_req: &PeeringRequest, peer_id: &PeerId, peerstore: &S) -> bool {
    if request::is_expired(peering_req.timestamp()) {
        false
    } else if !peer::is_verified(peerstore.last_verification_response(&peer_id).unwrap_or(0)) {
        false
    } else if salt::is_expired(peering_req.salt_expiration_time()) {
        false
    } else {
        true
    }
}

fn handle_peering_request(_: &PeeringRequest, msg_bytes: &[u8], server_tx: &ServerTx, source_addr: SocketAddr) {
    let request_hash = &hash::sha256(&msg_bytes)[..];

    reply_with_peering_response(request_hash, &server_tx, source_addr);
}

fn reply_with_peering_response(request_hash: &[u8], tx: &ServerTx, source_addr: SocketAddr) {
    todo!()
}

fn validate_peering_response(peering_res: &PeeringResponse, peer_id: &PeerId, request_mngr: &RequestManager) -> bool {
    if let Some(request_hash) = request_mngr.get_request_hash::<PeeringRequest>(peer_id) {
        peering_res.request_hash() == &request_hash
    } else {
        false
    }
}

fn handle_peering_response(peering_res: &PeeringResponse) {
    // hive.go: PeeringResponse messages are handled in the handleReply function of the validation
    todo!("handle_peering_response")
}

fn validate_drop_request(drop_req: &DropRequest) -> bool {
    request::is_expired(drop_req.timestamp())
}

fn handle_drop_request(
    _: &DropRequest,
    peer_id: PeerId,
    inbound_nh: &mut InboundNeighborhood,
    outbound_nh: &mut OutboundNeighborhood,
    rejection_filter: &mut RejectionFilter,
    server_tx: &ServerTx,
    source_addr: SocketAddr,
) {
    let mut maybe_dropped_peer = inbound_nh.remove_peer(&peer_id);

    if let Some(dropped_peer) = outbound_nh.remove_peer(&peer_id) {
        maybe_dropped_peer.replace(dropped_peer);

        rejection_filter.include_peer(peer_id);

        // ```go
        //     // if not yet updating, trigger an immediate update
        //     if updateOutResultChan == nil && updateTimer.Stop() {
        //         updateTimer.Reset(0)
        // ```
    }

    if maybe_dropped_peer.is_some() {
        reply_with_drop_request(server_tx, source_addr);
    }
}

// Replies to a drop request with a drop request.
fn reply_with_drop_request(server_tx: &ServerTx, target_addr: SocketAddr) {
    let drop_req_bytes = DropRequest::new()
        .to_protobuf()
        .expect("error encoding drop request")
        .to_vec();

    server_tx.send(OutgoingPacket {
        msg_type: MessageType::DropRequest,
        msg_bytes: drop_req_bytes,
        target_socket_addr: target_addr,
    });
}

fn update_salts(
    local: &Local,
    filter: &mut RejectionFilter,
    drop_neighbors_on_salt_update: bool,
    inbound: &mut InboundNeighborhood,
    outbound: &mut OutboundNeighborhood,
    packet_tx: &ServerTx,
    event_tx: &PeeringEventTx,
) {
    // Create and set new private and public salts for the local peer.
    let private_salt = Salt::default();
    let private_salt_exp_time = private_salt.expiration_time();
    let public_salt = Salt::default();
    let public_salt_exp_time = public_salt.expiration_time();

    local.set_private_salt(private_salt);
    local.set_public_salt(public_salt);

    // Clear the rejection filter.
    todo!("clear rejection filter");
    // filter.clear_peers();

    // Either drop, or update the neighborhoods.
    if drop_neighbors_on_salt_update {
        drop_neighborhood(inbound as &InboundNeighborhood, packet_tx);
        drop_neighborhood(outbound as &OutboundNeighborhood, packet_tx);

        inbound.clear();
        outbound.clear();
    } else {
        inbound.update_distances();
        outbound.update_distances();
    }

    log::debug!(
        "Salts updated: Public: {}, Private: {}",
        public_salt_exp_time,
        private_salt_exp_time
    );

    // Fire 'SaltUpdated' event.
    event_tx.send(PeeringEvent::SaltUpdated);
}

fn drop_neighborhood<'a, Nh>(neighborhood: &'a Nh, server_tx: &ServerTx)
where
    &'a Nh: IntoIterator<Item = Peer, IntoIter = std::vec::IntoIter<Peer>>,
{
    for peer in neighborhood {
        send_drop_request(peer, server_tx);
    }
}

// Initiates a drop request.
fn send_drop_request(peer: Peer, server_tx: &ServerTx) {
    let drop_req_bytes = DropRequest::new()
        .to_protobuf()
        .expect("error encoding drop request")
        .to_vec();

    let port = peer
        .services()
        .get(AUTOPEERING_SERVICE_NAME)
        .expect("invalid autopeering peer")
        .port();

    server_tx.send(OutgoingPacket {
        msg_type: MessageType::DropRequest,
        msg_bytes: drop_req_bytes,
        target_socket_addr: SocketAddr::new(peer.ip_address(), port),
    });
}
