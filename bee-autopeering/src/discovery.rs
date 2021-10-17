// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use tokio::sync::mpsc;

use crate::{
    config::AutopeeringConfig,
    delay::{Delay, DelayBuilder, DelayMode, Repeat as _},
    discovery_messages::{DiscoveryRequest, VerificationRequest, VerificationResponse},
    hash,
    identity::{LocalId, PeerId},
    multiaddr::AutopeeringMultiaddr,
    packet::{IncomingPacket, MessageType, OutgoingPacket},
    peer::Peer,
    request::RequestManager,
    server::ServerSocket,
    service_map::ServiceMap,
    time,
};

use std::{collections::VecDeque, fmt, net::SocketAddr, ops::DerefMut, time::Duration};

type BootstrapPeer = Peer;

// hive.go: time interval after which the next peer is reverified
const DEFAULT_REVERIFY_INTERVAL_SECS: u64 = 10;
// hive.go: time interval after which peers are queried for new peers
const DEFAULT_QUERY_INTERVAL_SECS: u64 = 60;
// hive.go: maximum number of peers that can be managed
const DEFAULT_MAX_MANAGED: usize = 1000;
// maximum number of peers kept in the replacement list
const DEFAULT_MAX_REPLACEMENTS: usize = 10;
// TODO:
const BACKOFF_INTERVALL_MILLISECS: u64 = 500;
// TODO:
const JITTER: f32 = 0.5;
//
const EXPONENTIAL_BACKOFF_FACTOR: f32 = 1.5;
// TODO:
const MAX_RETRIES: usize = 2;
// hive.go: PingExpiration is the time until a peer verification expires (12 hours)
pub(crate) const PING_EXPIRATION: u64 = 12 * 60 * 60;
// hive.go: MaxPeersInResponse is the maximum number of peers returned in DiscoveryResponse.
const MAX_PEERS_IN_RESPONSE: usize = 6;
// hive.go: MaxServices is the maximum number of services a peer can support.

pub(crate) struct DiscoveredPeer {
    peer: Peer,
    // how often that peer has been re-verified
    verified_count: usize,
    // number of returned new peers when queried the last time
    last_new_peers: usize,
}

impl DiscoveredPeer {
    pub fn peer(&self) -> &Peer {
        &self.peer
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer.peer_id()
    }

    pub fn verified_count(&self) -> usize {
        self.verified_count
    }

    pub fn last_new_peers(&self) -> usize {
        self.last_new_peers
    }
}

impl From<Peer> for DiscoveredPeer {
    fn from(peer: Peer) -> Self {
        Self {
            peer,
            verified_count: 0,
            last_new_peers: 0,
        }
    }
}

impl Into<Peer> for DiscoveredPeer {
    fn into(self) -> Peer {
        self.peer
    }
}

impl fmt::Debug for DiscoveredPeer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiscoveredPeer")
            .field("peer", &self.peer)
            .field("verified_count", &self.verified_count)
            .field("last_new_peers", &self.last_new_peers)
            .finish()
    }
}

pub(crate) struct DiscoveredPeerlist {
    peers: VecDeque<DiscoveredPeer>,
}

impl DiscoveredPeerlist {
    pub fn new() -> Self {
        Self {
            peers: VecDeque::with_capacity(DEFAULT_MAX_MANAGED),
        }
    }

    pub fn remove(&mut self, peer_id: PeerId) {
        if let Some(index) = self.peers.iter().position(|peer| peer.peer_id() == peer_id) {
            self.remove_at(index)
        }
    }

    pub fn remove_at(&mut self, index: usize) {
        self.peers.remove(index);
    }

    pub fn push_front(&mut self, peer: DiscoveredPeer) {
        self.peers.push_front(peer)
    }

    pub fn push_back(&mut self, peer: DiscoveredPeer) {
        self.peers.push_back(peer)
    }

    pub fn pop_front(&mut self) {
        self.peers.pop_front();
    }

    pub fn pop_back(&mut self) {
        self.peers.pop_back();
    }
}

/// Discovery related events.
#[derive(Debug)]
pub enum DiscoveryEvent {
    /// A new peer has been discovered.
    PeerDiscovered { peer: Peer },
    /// A peer has been deleted (e.g. due to a failed re-verification).
    PeerDeleted { peer_id: PeerId },
}

/// Exposes peer discovery related events.
pub type DiscoveryEventRx = mpsc::UnboundedReceiver<DiscoveryEvent>;
type DiscoveryEventTx = mpsc::UnboundedSender<DiscoveryEvent>;

fn event_chan() -> (DiscoveryEventTx, DiscoveryEventRx) {
    mpsc::unbounded_channel::<DiscoveryEvent>()
}

pub(crate) struct DiscoveryManagerConfig {
    pub(crate) entry_nodes: Vec<AutopeeringMultiaddr>,
    pub(crate) entry_nodes_prefer_ipv6: bool,
    pub(crate) run_as_entry_node: bool,
    pub(crate) version: u32,
    pub(crate) network_id: u32,
    pub(crate) source_addr: SocketAddr,
    pub(crate) services: ServiceMap,
}

impl DiscoveryManagerConfig {
    pub fn new(config: &AutopeeringConfig, version: u32, network_id: u32, services: ServiceMap) -> Self {
        Self {
            entry_nodes: config.entry_nodes.clone(),
            entry_nodes_prefer_ipv6: config.entry_nodes_prefer_ipv6,
            run_as_entry_node: config.run_as_entry_node,
            version,
            network_id,
            source_addr: config.bind_addr,
            services,
        }
    }
}

pub(crate) struct DiscoveryManager {
    // Config.
    config: DiscoveryManagerConfig,
    // The local id to sign outgoing packets.
    local_id: LocalId,
    // Channel halfs for sending/receiving discovery related packets.
    socket: ServerSocket,
    // Handles requests.
    request_mngr: RequestManager,
    // Publishes discovery related events.
    event_tx: DiscoveryEventTx,
}

impl DiscoveryManager {
    pub(crate) fn new(
        config: DiscoveryManagerConfig,
        local_id: LocalId,
        socket: ServerSocket,
        request_mngr: RequestManager,
    ) -> (Self, DiscoveryEventRx) {
        let (event_tx, event_rx) = event_chan();
        (
            Self {
                config,
                local_id,
                socket,
                request_mngr,
                event_tx,
            },
            event_rx,
        )
    }

    pub(crate) async fn run(self) {
        let DiscoveryManager {
            config,
            local_id,
            socket,
            request_mngr,
            event_tx,
        } = self;

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
                    MessageType::VerificationRequest => {
                        let verif_req = VerificationRequest::from_protobuf(&msg_bytes).expect("error decoding ping");

                        // Validate the received ping.
                        if !validate_verification_request(&verif_req, config.version, config.network_id) {
                            log::debug!("Received invalid Ping: {:?}", verif_req);
                            continue;
                        }

                        handle_verification_request();
                    }
                    MessageType::VerificationResponse => {
                        let pong = VerificationResponse::from_protobuf(&msg_bytes).expect("error decoding pong");

                        if !validate_verification_response(&pong, &[0u8]) {
                            log::debug!("Received invalid Pong: {:?}", pong);
                            continue;
                        }

                        handle_verification_response();
                    }
                    MessageType::DiscoveryRequest => {
                        let disc_req =
                            DiscoveryRequest::from_protobuf(&msg_bytes).expect("error decoding discover request");
                    }
                    MessageType::DiscoveryResponse => {}
                    _ => panic!("unsupported messasge type"),
                }
            }
        }

        // // Send a `Ping` to all entry nodes.
        // let bootstrap_peers = config
        //     .entry_nodes
        //     .iter()
        //     .map(|addr| addr.host_socketaddr())
        //     .collect::<Vec<_>>();

        // for target_addr in bootstrap_peers {
        //     let ping = ping_factory.make(target_addr.ip());
        //     let msg_bytes = ping.protobuf().expect("error encoding ping");
        //     let signature = local_id.sign(&msg_bytes);
        //     let packet = Packet::new(MessageType::Ping, &msg_bytes, &local_id.public_key(), signature);
        //     // socket.send(OutgoingPacket { packet, target_addr });
        // }
    }
}

fn send_verification_request() {
    todo!()
}

fn validate_verification_request(ping: &VerificationRequest, version: u32, network_id: u32) -> bool {
    if ping.version() != version {
        false
    } else if ping.network_id() != network_id {
        false
    } else if ping.timestamp() < time::unix_now() - PING_EXPIRATION {
        false
    } else {
        true
    }
}

fn handle_verification_request() {
    // // Send back a corresponding pong.
    // let ping_hash = hash::sha256(&msg_bytes).to_vec();
    // let pong = VerificationResponse::new(ping_hash, config.services.clone(), source_addr.ip());
    // let pong_bytes = pong.protobuf().expect("error encoding pong").to_vec();

    // tx.send(OutgoingPacket {
    //     msg_type: MessageType::VerificationResponse,
    //     msg_bytes: pong_bytes,
    //     target_addr: source_addr,
    // })
    // .expect("error sending pong to server");
    todo!()
}

fn send_verification_response() {
    todo!()
}

fn validate_verification_response(pong: &VerificationResponse, expected_ping_hash: &[u8]) -> bool {
    pong.request_hash() == expected_ping_hash
}

fn handle_verification_response() {
    // // Try to find the corresponding 'Ping', that we sent to that peer.
    // if let Some(ping) = request_mngr.get_request::<VerificationRequest>(peer_id.clone()) {
    //     let expected_ping_hash = hash::sha256(&ping.protobuf().expect("error encoding ping"));
    //     if !validate_verification_response(&pong, &expected_ping_hash) {
    //         log::debug!("Received invalid pong");
    //         continue;
    //     }
    // } else {
    //     log::debug!(
    //         "Received 'Pong' from {}, but 'Ping' was never sent, or has already expired.",
    //         peer_id
    //     );
    //     continue;
    // }
    todo!()
}
