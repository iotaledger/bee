// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use tokio::sync::mpsc;

use crate::{
    config::AutopeeringConfig,
    delay::{Delay, DelayBuilder, DelayMode, Repeat as _},
    discovery_messages::{DiscoveryRequest, DiscoveryResponse, VerificationRequest, VerificationResponse},
    hash,
    identity::PeerId,
    local::Local,
    multiaddr::{AddressKind, AutopeeringMultiaddr},
    packet::{msg_hash, IncomingPacket, MessageType, OutgoingPacket},
    peer::{self, Peer},
    peerstore::{self, PeerStore},
    request::{self, RequestManager},
    server::{marshal, ServerSocket, ServerTx},
    service_map::{ServiceMap, ServiceTransport, AUTOPEERING_SERVICE_NAME},
    shutdown::ShutdownRx,
    time,
};

use std::{
    collections::{HashSet, VecDeque},
    fmt,
    net::SocketAddr,
    ops::DerefMut,
    time::Duration,
};

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
// hive.go: is the time until a peer verification expires (12 hours)
pub(crate) const VERIFICATION_EXPIRATION: u64 = 12 * 60 * 60;
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
}

impl DiscoveryManagerConfig {
    pub fn new(config: &AutopeeringConfig, version: u32, network_id: u32) -> Self {
        Self {
            entry_nodes: config.entry_nodes.clone(),
            entry_nodes_prefer_ipv6: config.entry_nodes_prefer_ipv6,
            run_as_entry_node: config.run_as_entry_node,
            version,
            network_id,
            source_addr: config.bind_addr,
        }
    }
}

pub(crate) struct DiscoveryManager<S> {
    // Config.
    config: DiscoveryManagerConfig,
    // The local id to sign outgoing packets.
    local: Local,
    // Channel halfs for sending/receiving discovery related packets.
    socket: ServerSocket,
    // Handles requests.
    request_mngr: RequestManager,
    // Publishes discovery related events.
    event_tx: DiscoveryEventTx,
    // The storage for discovered peers.
    peerstore: S,
    //
    active_peers: HashSet<Peer>,
    replacement_peers: HashSet<Peer>,
    // The shutdown signal receiver.
    shutdown_rx: ShutdownRx,
}

impl<S: PeerStore> DiscoveryManager<S> {
    pub(crate) fn new(
        config: DiscoveryManagerConfig,
        local: Local,
        socket: ServerSocket,
        request_mngr: RequestManager,
        peerstore: S,
        shutdown_rx: ShutdownRx,
    ) -> (Self, DiscoveryEventRx) {
        let (event_tx, event_rx) = event_chan();
        (
            Self {
                config,
                local,
                socket,
                request_mngr,
                event_tx,
                peerstore,
                active_peers: HashSet::default(),
                replacement_peers: HashSet::default(),
                shutdown_rx,
            },
            event_rx,
        )
    }

    pub(crate) async fn run(self) {
        let DiscoveryManager {
            config,
            local,
            socket,
            request_mngr,
            event_tx,
            peerstore,
            active_peers,
            replacement_peers,
            mut shutdown_rx,
        } = self;

        let DiscoveryManagerConfig {
            mut entry_nodes,
            entry_nodes_prefer_ipv6,
            run_as_entry_node,
            version,
            network_id,
            source_addr,
        } = config;

        let ServerSocket {
            mut server_rx,
            server_tx,
        } = socket;

        // Send verification and discovery request to the entry nodes.
        contact_entry_nodes(&mut entry_nodes, entry_nodes_prefer_ipv6, &request_mngr, &server_tx).await;

        loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    break;
                }
                o = server_rx.recv() => {
                    if let Some(IncomingPacket {
                        msg_type,
                        msg_bytes,
                        source_addr,
                        peer_id,
                    }) = o
                    {
                        match msg_type {
                            MessageType::VerificationRequest => {
                                log::debug!("Received verification request.");

                                let verif_req = VerificationRequest::from_protobuf(&msg_bytes)
                                    .expect("error decoding verification request");

                                if !validate_verification_request(&verif_req, version, network_id) {
                                    log::debug!("Received invalid verification request: {:?}", verif_req);
                                    continue;
                                }
                                log::debug!("Received valid verification request: {:?}", verif_req);

                                handle_verification_request(
                                    &verif_req,
                                    &peer_id,
                                    &msg_bytes,
                                    &server_tx,
                                    source_addr,
                                    &local,
                                    &peerstore,
                                    &request_mngr,
                                );
                            }
                            MessageType::VerificationResponse => {
                                log::debug!("Received verification response.");

                                let verif_res = VerificationResponse::from_protobuf(&msg_bytes)
                                    .expect("error decoding verification response");

                                if !validate_verification_response(&verif_res, &request_mngr, &peer_id) {
                                    log::debug!("Received invalid verification response: {:?}", verif_res);
                                    continue;
                                }

                                handle_verification_response(
                                    &verif_res,
                                    &peer_id,
                                    &msg_bytes,
                                    &server_tx,
                                    source_addr,
                                    &local,
                                    &peerstore,
                                    &request_mngr,
                                );
                            }
                            MessageType::DiscoveryRequest => {
                                log::debug!("Received discovery request.");

                                let disc_req =
                                    DiscoveryRequest::from_protobuf(&msg_bytes).expect("error decoding discover request");

                                if !validate_discovery_request(&disc_req) {
                                    log::debug!("Received invalid discovery request: {:?}", disc_req);
                                    continue;
                                }

                                let request_hash = &hash::sha256(&msg_bytes)[..];

                                send_discovery_response(request_hash, &server_tx, source_addr);
                            }
                            MessageType::DiscoveryResponse => {
                                log::debug!("Received discovery response.");

                                let disc_res =
                                    DiscoveryResponse::from_protobuf(&msg_bytes).expect("error decoding discovery response");

                                if !validate_discovery_response(&disc_res, &request_mngr, &peer_id) {
                                    log::debug!("Received invalid discovery response: {:?}", disc_res);
                                    continue;
                                }

                                handle_discovery_response();
                            }
                            _ => panic!("unsupported discovery message type"),
                        }
                    }
                }
            }
        }
    }
}

async fn contact_entry_nodes(
    entry_nodes: &mut Vec<AutopeeringMultiaddr>,
    entry_nodes_prefer_ipv6: bool,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
) {
    log::debug!("Contacting entry nodes.");

    // Send verification request to entry nodes
    for mut entry_addr in entry_nodes {
        let entry_socketaddr = match entry_addr.address_kind() {
            AddressKind::Ip4 | AddressKind::Ip6 => {
                // Unwrap: for those address kinds the returned option is always `Some`.
                entry_addr.socket_addr().unwrap()
            }
            AddressKind::Dns => {
                if entry_addr.resolve_dns().await {
                    let entry_socketaddrs = entry_addr.resolved_addrs();
                    let has_ip4 = entry_socketaddrs.iter().position(|s| s.is_ipv4());
                    let has_ip6 = entry_socketaddrs.iter().position(|s| s.is_ipv6());

                    match (has_ip4, has_ip6) {
                        // Only IP4 or only IP6
                        (Some(index), None) | (None, Some(index)) => entry_socketaddrs[index],
                        // Both are available
                        (Some(index1), Some(index2)) => {
                            if entry_nodes_prefer_ipv6 {
                                entry_socketaddrs[index2]
                            } else {
                                entry_socketaddrs[index1]
                            }
                        }
                        // Both being None is not possible.
                        _ => unreachable!(),
                    }
                } else {
                    // Ignore that entry node.
                    continue;
                }
            }
        };

        let mut peer = Peer::new(entry_socketaddr.ip(), entry_addr.public_key().clone());
        peer.add_service(AUTOPEERING_SERVICE_NAME, ServiceTransport::Udp, entry_socketaddr.port());

        send_verification_request(&peer, &request_mngr, &server_tx);
    }
}

fn validate_verification_request(verif_req: &VerificationRequest, version: u32, network_id: u32) -> bool {
    log::debug!("Validating verification request.");

    if verif_req.version() != version {
        false
    } else if verif_req.network_id() != network_id {
        false
    } else if request::is_expired(verif_req.timestamp()) {
        false
    } else {
        // NOTE: the validity of the transmitted source and target addresses is ensured through the
        // `VerificationRequest` type.
        true
    }
}

fn handle_verification_request<S: PeerStore>(
    verif_req: &VerificationRequest,
    peer_id: &PeerId,
    msg_bytes: &[u8],
    server_tx: &ServerTx,
    source_addr: SocketAddr,
    local: &Local,
    peerstore: &S,
    request_mngr: &RequestManager,
) {
    log::debug!("Handling verification request.");

    peerstore.update_last_verification_request(peer_id.clone());

    reply_with_verification_response(verif_req, msg_bytes, &server_tx, &local, source_addr);

    // ```go
    // if the peer is unknown or expired, send a Ping to verify
    // if !p.IsVerified(from.ID(), dstAddr.IP) {
    //     p.sendPing(dstAddr, from.ID())
    // } else if !p.mgr.isKnown(from.ID()) {
    //     // add a discovered peer to the manager if it is new but verified
    // 	   p.mgr.addDiscoveredPeer(newPeer(from, s.LocalAddr().Network(), dstAddr))
    // }
    // ```
    if let Some(last_verif_res) = peerstore.last_verification_response(peer_id) {
        if !peer::is_verified(last_verif_res) {
            reply_with_verification_request(peer_id, request_mngr, server_tx, source_addr);
        }
    } else {
        reply_with_verification_request(peer_id, request_mngr, server_tx, source_addr);
    }
}

fn reply_with_verification_response(
    verif_req: &VerificationRequest,
    msg_bytes: &[u8],
    tx: &ServerTx,
    local: &Local,
    target_addr: SocketAddr,
) {
    log::debug!("Replying with verification response.");

    let request_hash = &msg_hash(MessageType::VerificationRequest, msg_bytes);

    let verif_res = VerificationResponse::new(request_hash, local.services(), target_addr.ip());
    let verif_res_bytes = verif_res
        .protobuf()
        .expect("error encoding verification response")
        .to_vec();

    // hive.go:
    // ```go
    // // the destination address uses the source IP address of the packet plus the src_port from the message
    // dstAddr := &net.UDPAddr{
    // 	IP:   fromAddr.IP,
    // 	Port: int(m.SrcPort),
    // }
    // ```
    tx.send(OutgoingPacket {
        msg_type: MessageType::VerificationResponse,
        msg_bytes: verif_res_bytes,
        target_addr: SocketAddr::new(target_addr.ip(), verif_req.source_addr.port()),
    })
    .expect("error sending verification response to server");
}

fn reply_with_verification_request(
    peer_id: &PeerId,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
    target_addr: SocketAddr,
) {
    log::debug!("Replying with verification request.");

    let verif_req_bytes = request_mngr
        .new_verification_request(peer_id.clone(), target_addr.ip())
        .protobuf()
        .expect("error encoding verification request")
        .to_vec();

    server_tx
        .send(OutgoingPacket {
            msg_type: MessageType::VerificationRequest,
            msg_bytes: verif_req_bytes,
            target_addr,
        })
        .expect("error sending verification request to server");
}

fn send_verification_request(target: &Peer, request_mngr: &RequestManager, server_tx: &ServerTx) {
    log::debug!("Sending verification request to: {:?}", target);

    let verif_req = request_mngr.new_verification_request(target.peer_id(), target.ip_address());
    let verif_req_bytes = verif_req
        .protobuf()
        .expect("error encoding verification request")
        .to_vec();

    let port = target
        .services()
        .port(AUTOPEERING_SERVICE_NAME)
        .expect("peer doesn't support autopeering");

    server_tx
        .send(OutgoingPacket {
            msg_type: MessageType::VerificationRequest,
            msg_bytes: verif_req_bytes,
            target_addr: SocketAddr::new(target.ip_address(), port),
        })
        .expect("error sending verification request to server");
}

fn validate_verification_response(
    verif_res: &VerificationResponse,
    request_mngr: &RequestManager,
    peer_id: &PeerId,
) -> bool {
    log::debug!("Validating verification response.");

    if let Some(request_hash) = request_mngr.get_request_hash::<VerificationRequest>(peer_id) {
        verif_res.request_hash() == &request_hash[..]
    } else {
        false
    }
}

fn handle_verification_response<S: PeerStore>(
    verif_res: &VerificationResponse,
    peer_id: &PeerId,
    msg_bytes: &[u8],
    server_tx: &ServerTx,
    source_addr: SocketAddr,
    local: &Local,
    peerstore: &S,
    request_mngr: &RequestManager,
) {
    log::debug!("Handling verification response.");

    peerstore.update_last_verification_response(peer_id.clone());

    // TEMP: on each valid verification response send a discovery request
    // reply_with_discovery_request(peer_id, request_mngr, server_tx, source_addr);
}

fn reply_with_discovery_request(
    peer_id: &PeerId,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
    target_addr: SocketAddr,
) {
    log::debug!("Replying with discovery request.");

    let disc_req_bytes = request_mngr
        .new_discovery_request(peer_id.clone(), target_addr.ip())
        .protobuf()
        .expect("error encoding discovery request")
        .to_vec();

    server_tx
        .send(OutgoingPacket {
            msg_type: MessageType::DiscoveryRequest,
            msg_bytes: disc_req_bytes,
            target_addr,
        })
        .expect("error sending discovery request to server");
}

fn send_discovery_request(target: &Peer, request_mngr: &RequestManager, server_tx: &ServerTx) {
    log::debug!("Sending discovery request to: {:?}", target);

    let disc_req = request_mngr.new_discovery_request(target.peer_id(), target.ip_address());
    let disc_req_bytes = disc_req
        .protobuf()
        .expect("error encoding verification request")
        .to_vec();

    let port = target
        .services()
        .port(AUTOPEERING_SERVICE_NAME)
        .expect("peer doesn't support autopeering");

    server_tx
        .send(OutgoingPacket {
            msg_type: MessageType::DiscoveryRequest,
            msg_bytes: disc_req_bytes,
            target_addr: SocketAddr::new(target.ip_address(), port),
        })
        .expect("error sending discovery request to server");
}

fn validate_discovery_request(disc_req: &DiscoveryRequest) -> bool {
    // validates an incoming discovery request
    todo!()
}

fn send_discovery_response(request_hash: &[u8], tx: &ServerTx, target_addr: SocketAddr) {
    // send a random set of peers as a response
    todo!()
}

fn validate_discovery_response(disc_res: &DiscoveryResponse, request_mngr: &RequestManager, peer_id: &PeerId) -> bool {
    // check the peers we received
    todo!()
}

fn handle_discovery_response() {
    // things we need to do after we received a valid response to our request
    todo!()
}
