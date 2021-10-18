// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    distance::{Neighborhood, SIZE_INBOUND, SIZE_OUTBOUND},
    hash,
    local::Local,
    packet::{IncomingPacket, MessageType, OutgoingPacket},
    peering_messages::{PeeringDrop, PeeringRequest, PeeringResponse},
    peerstore::{InMemoryPeerStore, PeerStore},
    request::RequestManager,
    salt::Salt,
    server::{OutgoingPacketTx, ServerSocket},
    service_map::AUTOPEERING_SERVICE_NAME,
    Peer, PeerId,
};

use tokio::sync::mpsc;

use std::{net::SocketAddr, time::Duration, vec};

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
    inbound_neighborhood: InboundNeighborhood,
    // Outbound neighborhood.
    outbound_neighborhood: OutboundNeighborhood,
}

impl<S: PeerStore> PeeringManager<S> {
    pub(crate) fn new(
        config: PeeringManagerConfig,
        local: Local,
        socket: ServerSocket,
        request_mngr: RequestManager,
        peerstore: S,
    ) -> (Self, PeeringEventRx) {
        let (event_tx, event_rx) = event_chan();

        let inbound_neighborhood = Neighborhood::new(local.clone());
        let outbound_neighborhood = Neighborhood::new(local.clone());

        (
            Self {
                config,
                local,
                socket,
                request_mngr,
                event_tx,
                peerstore,
                inbound_neighborhood,
                outbound_neighborhood,
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
            inbound_neighborhood,
            outbound_neighborhood,
        } = self;

        let PeeringManagerConfig {
            version,
            network_id,
            source_addr,
            drop_neighbors_on_salt_update,
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

fn update_salts(
    local: &Local,
    filter: &mut Filter,
    drop_neighbors_on_salt_update: bool,
    inbound: &mut InboundNeighborhood,
    outbound: &mut OutboundNeighborhood,
    packet_tx: &OutgoingPacketTx,
    event_tx: &PeeringEventTx,
) {
    // Create and set new private and public salts for the local peer.
    let private_salt = Salt::default();
    let private_salt_exp_time = private_salt.expiration_time();
    let public_salt = Salt::default();
    let public_salt_exp_time = public_salt.expiration_time();

    local.set_private_salt(private_salt);
    local.set_public_salt(public_salt);

    // Clean the rejection filter.
    filter.clean();

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

fn drop_neighborhood<'a, Nh>(neighborhood: &'a Nh, packet_tx: &OutgoingPacketTx)
where
    &'a Nh: IntoIterator<Item = Peer, IntoIter = std::vec::IntoIter<Peer>>,
{
    for peer in neighborhood {
        let peering_drop_bytes = PeeringDrop::new()
            .protobuf()
            .expect("error encoding PeeringDrop message")
            .to_vec();

        let port = peer
            .services()
            .port(AUTOPEERING_SERVICE_NAME)
            .expect("invalid autopeering peer");

        packet_tx.send(OutgoingPacket {
            msg_type: MessageType::PeeringDrop,
            msg_bytes: peering_drop_bytes,
            target_addr: SocketAddr::new(peer.ip_address(), port),
        });
    }
}

struct Filter {}
impl Filter {
    fn clean(&mut self) {}
}
