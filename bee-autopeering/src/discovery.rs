// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    delay::{DelayFactory, DelayFactoryBuilder, DelayFactoryMode, DelayedRepeat as _},
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
    service_map::{ServiceMap, ServicePort, ServiceTransport, AUTOPEERING_SERVICE_NAME},
    shutdown::{Runnable, ShutdownRx},
    time,
};

use tokio::sync::mpsc;

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
// hive.go: maximum number of peers kept in the replacement list
const DEFAULT_MAX_REPLACEMENTS: usize = 10;
// The default delay between requests to a single peer.
const BACKOFF_INTERVALL_MILLISECS: u64 = 500;
// A factor that determines the range from which a concrete delay is picked randomly.
const JITTER: f32 = 0.5;
// A factor that determines the intervall lengths between repeated requests to a peer.
const EXPONENTIAL_BACKOFF_FACTOR: f32 = 1.5;
// The number of times a request is repeated in case the peer doesn't reply.
const MAX_RETRIES: usize = 2;
// hive.go: is the time until a peer verification expires (12 hours)
pub(crate) const VERIFICATION_EXPIRATION: u64 = 12 * 60 * 60;
// hive.go: MaxPeersInResponse is the maximum number of peers returned in DiscoveryResponse.
const MAX_PEERS_IN_RESPONSE: usize = 6;
// hive.go: MaxServices is the maximum number of services a peer can support.
const MAX_SERVICES: usize = 5;

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
    // TODO
    active_peers: HashSet<Peer>,
    // TODO
    replacement_peers: HashSet<Peer>,
}

impl<S: PeerStore> DiscoveryManager<S> {
    pub(crate) fn new(
        config: DiscoveryManagerConfig,
        local: Local,
        socket: ServerSocket,
        request_mngr: RequestManager,
        peerstore: S,
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
            },
            event_rx,
        )
    }
}

#[async_trait::async_trait]
impl<S: PeerStore> Runnable for DiscoveryManager<S> {
    const NAME: &'static str = "DiscoveryManager";

    type Cancel = ShutdownRx;

    async fn run(self, mut shutdown_rx: Self::Cancel) {
        let DiscoveryManager {
            config,
            local,
            socket,
            request_mngr,
            event_tx,
            peerstore,
            active_peers,
            replacement_peers,
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
        add_entry_nodes(
            &mut entry_nodes,
            entry_nodes_prefer_ipv6,
            &request_mngr,
            &server_tx,
            &peerstore,
        )
        .await;

        'recv: loop {
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
                        let ctx = HandlerContext {
                            peer_id: &peer_id,
                            msg_bytes: &msg_bytes,
                            server_tx: &server_tx,
                            local: &local,
                            peerstore: &peerstore,
                            request_mngr: &request_mngr,
                            source_addr,
                            event_tx: &event_tx,
                        };

                        match msg_type {
                            MessageType::VerificationRequest => {
                                let verif_req = if let Ok(verif_req) = VerificationRequest::from_protobuf(&msg_bytes) {
                                    verif_req
                                } else {
                                    log::debug!("Error decoding verification request from {}.", &peer_id);
                                    continue 'recv;
                                };

                                if let Err(e) = validate_verification_request(&verif_req, version, network_id) {
                                    log::debug!("Received invalid verification request from {}. Reason: {:?}", &peer_id, e);
                                    continue 'recv;
                                } else {
                                    log::debug!("Received valid verification request from {}.", &peer_id);
                                    handle_verification_request(&verif_req, ctx);
                                }
                            }
                            MessageType::VerificationResponse => {
                                let verif_res = if let Ok(verif_res) = VerificationResponse::from_protobuf(&msg_bytes) {
                                    verif_res
                                } else {
                                    log::debug!("Error decoding verification response from {}.", &peer_id);
                                    continue 'recv;
                                };

                                if let Err(e) = validate_verification_response(&verif_res, &request_mngr, &peer_id, source_addr) {
                                    log::debug!("Received invalid verification response from {}. Reason: {:?}", &peer_id, e);
                                    continue 'recv;
                                } else {
                                    log::debug!("Received valid verification response from {}.", &peer_id);
                                    handle_verification_response(&verif_res, ctx);
                                }
                            }
                            MessageType::DiscoveryRequest => {
                                let disc_req = if let Ok(disc_req) = DiscoveryRequest::from_protobuf(&msg_bytes) {
                                    disc_req
                                } else {
                                    log::debug!("Error decoding discovery request from {}.", &peer_id);
                                    continue 'recv;
                                };

                                if let Err(e) = validate_discovery_request(&disc_req) {
                                    log::debug!("Received invalid discovery request from {}. Reason: {:?}", &peer_id, e);
                                    continue 'recv;
                                } else {
                                    log::debug!("Received valid discovery request from {}.", &peer_id);
                                    handle_discovery_request(&disc_req, ctx);
                                }
                            }
                            MessageType::DiscoveryResponse => {
                                let disc_res = if let Ok(disc_res) = DiscoveryResponse::from_protobuf(&msg_bytes) {
                                    disc_res
                                } else {
                                    log::debug!("Error decoding discovery response from {}.", &peer_id);
                                    continue 'recv;
                                };

                                if let Err(e) = validate_discovery_response(&disc_res, &request_mngr, &peer_id) {
                                    log::debug!("Received invalid discovery response from {}. Reason: {:?}", &peer_id, e);
                                    continue 'recv;
                                } else {
                                    log::debug!("Received valid discovery response from {}.", &peer_id);
                                    handle_discovery_response(&disc_res, ctx);
                                }
                            }
                            _ => log::debug!("Received unsupported discovery message type"),
                        }
                    }
                }
            }
        }
    }
}

async fn add_entry_nodes<S: PeerStore>(
    entry_nodes: &mut Vec<AutopeeringMultiaddr>,
    entry_nodes_prefer_ipv6: bool,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
    peerstore: &S,
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

        peerstore.insert_peer(peer);
        // send_verification_request(&peer, &request_mngr, &server_tx);
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// MESSAGE VALIDATION
///////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub(crate) enum ValidationError {
    // The protocol version must match.
    VersionMismatch {
        expected: u32,
        received: u32,
    },
    // The network id must match.
    NetworkIdMismatch {
        expected: u32,
        received: u32,
    },
    // The request must not be expired.
    RequestExpired,
    // The response must arrive in time.
    ResponseTimeout,
    // The hash of the corresponding request must be correct.
    IncorrectRequestHash,
    // The peer must have an autopeering service.
    NoAutopeeringService,
    // The service port must match with the detected port.
    ServicePortMismatch {
        expected: ServicePort,
        received: ServicePort,
    },
}

fn validate_verification_request(
    verif_req: &VerificationRequest,
    version: u32,
    network_id: u32,
) -> Result<(), ValidationError> {
    use ValidationError::*;

    if verif_req.version() != version {
        Err(VersionMismatch {
            expected: version,
            received: verif_req.version(),
        })
    } else if verif_req.network_id() != network_id {
        Err(NetworkIdMismatch {
            expected: network_id,
            received: verif_req.network_id(),
        })
    } else if request::is_expired(verif_req.timestamp()) {
        Err(RequestExpired)
    } else {
        // NOTE: the validity of the transmitted source and target addresses is ensured through the
        // `VerificationRequest` type.
        // TODO: maybe add check whether the peer sent the correct source address in the packet.
        // TODO: store own external IP address as perceived by the peer
        Ok(())
    }
}

fn validate_verification_response(
    verif_res: &VerificationResponse,
    request_mngr: &RequestManager,
    peer_id: &PeerId,
    source_addr: SocketAddr,
) -> Result<(), ValidationError> {
    use ValidationError::*;

    if let Some(request_hash) = request_mngr.get_request_hash::<VerificationRequest>(peer_id) {
        if verif_res.request_hash() == &request_hash[..] {
            let services = verif_res.services();
            if let Some(autopeering) = services.get(AUTOPEERING_SERVICE_NAME) {
                if autopeering.port() == source_addr.port() {
                    Ok(())
                } else {
                    Err(ServicePortMismatch {
                        expected: autopeering.port(),
                        received: source_addr.port(),
                    })
                }
            } else {
                Err(NoAutopeeringService)
            }
        } else {
            Err(IncorrectRequestHash)
        }
    } else {
        Err(ResponseTimeout)
    }
}

fn validate_discovery_request(disc_req: &DiscoveryRequest) -> Result<(), ValidationError> {
    use ValidationError::*;

    if request::is_expired(disc_req.timestamp()) {
        Err(RequestExpired)
    } else {
        Ok(())
    }
}

fn validate_discovery_response(
    disc_res: &DiscoveryResponse,
    request_mngr: &RequestManager,
    peer_id: &PeerId,
) -> Result<(), ValidationError> {
    use ValidationError::*;

    if let Some(request_hash) = request_mngr.get_request_hash::<DiscoveryRequest>(peer_id) {
        if disc_res.request_hash() == &request_hash[..] {
            for peer in disc_res.peers() {
                // TODO: consider performing some checks on the peers we received, for example:
                // * does the peer have necessary services (autopeering, gossip, fpc, ...)
                // * is the ip address valid (not a 0.0.0.0, etc)
            }
            Ok(())
        } else {
            Err(IncorrectRequestHash)
        }
    } else {
        Err(ResponseTimeout)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// MESSAGE HANDLING
///////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct HandlerContext<'a, S: PeerStore> {
    peer_id: &'a PeerId,
    msg_bytes: &'a [u8],
    server_tx: &'a ServerTx,
    local: &'a Local,
    peerstore: &'a S,
    request_mngr: &'a RequestManager,
    source_addr: SocketAddr,
    event_tx: &'a DiscoveryEventTx,
}

fn handle_verification_request<S: PeerStore>(verif_req: &VerificationRequest, ctx: HandlerContext<S>) {
    log::debug!("Handling verification request.");

    ctx.peerstore.update_last_verification_request(ctx.peer_id.clone());

    reply_with_verification_response(verif_req, ctx.msg_bytes, ctx.server_tx, ctx.local, ctx.source_addr);

    // ```go
    // if the peer is unknown or expired, send a Ping to verify
    // if !p.IsVerified(from.ID(), dstAddr.IP) {
    //     p.sendPing(dstAddr, from.ID())
    // } else if !p.mgr.isKnown(from.ID()) {
    //     // add a discovered peer to the manager if it is new but verified
    // 	   p.mgr.addDiscoveredPeer(newPeer(from, s.LocalAddr().Network(), dstAddr))
    // }
    // ```
    if let Some(last_verif_res) = ctx.peerstore.last_verification_response(ctx.peer_id) {
        if !peer::is_verified(last_verif_res) {
            reply_with_verification_request(ctx.peer_id, ctx.request_mngr, ctx.server_tx, ctx.source_addr);
        }
    } else {
        reply_with_verification_request(ctx.peer_id, ctx.request_mngr, ctx.server_tx, ctx.source_addr);
    }
}

fn handle_verification_response<S: PeerStore>(verif_res: &VerificationResponse, ctx: HandlerContext<S>) {
    log::debug!("Handling verification response.");

    // Remove the corresponding request from the request manager.
    ctx.request_mngr.remove_request::<VerificationRequest>(ctx.peer_id);

    ctx.peerstore.update_last_verification_response(ctx.peer_id.clone());

    // TEMP: on each valid verification response send a discovery request
    // reply_with_discovery_request(peer_id, request_mngr, server_tx, source_addr);
}

fn handle_discovery_request<S: PeerStore>(disc_req: &DiscoveryRequest, ctx: HandlerContext<S>) {
    log::debug!("Handling discovery request.");

    reply_with_discovery_response(disc_req, ctx.msg_bytes, ctx.server_tx, ctx.local, ctx.source_addr);
}

fn handle_discovery_response<S: PeerStore>(disc_res: &DiscoveryResponse, ctx: HandlerContext<S>) {
    log::debug!("Handling discovery response.");

    // TODO: store the discovered peers; fire `PeerDiscovered` event.
    for peer in disc_res.peers() {
        log::debug!("{:?}", peer);
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// REPLYING
///////////////////////////////////////////////////////////////////////////////////////////////////////////

fn reply_with_verification_request(
    peer_id: &PeerId,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
    target_addr: SocketAddr,
) {
    log::debug!("Replying with verification request.");

    let verif_req_bytes = request_mngr
        .new_verification_request(peer_id.clone(), target_addr.ip())
        .to_protobuf()
        .expect("error encoding verification request")
        .to_vec();

    server_tx
        .send(OutgoingPacket {
            msg_type: MessageType::VerificationRequest,
            msg_bytes: verif_req_bytes,
            target_socket_addr: target_addr,
        })
        .expect("error sending verification request to server");
}

fn reply_with_verification_response(
    verif_req: &VerificationRequest,
    msg_bytes: &[u8],
    tx: &ServerTx,
    local: &Local,
    target_addr: SocketAddr,
) {
    log::debug!("Replying with verification response.");

    let request_hash = msg_hash(MessageType::VerificationRequest, msg_bytes).to_vec();

    let verif_res = VerificationResponse::new(request_hash, local.services(), target_addr.ip());
    let verif_res_bytes = verif_res
        .to_protobuf()
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
        target_socket_addr: SocketAddr::new(target_addr.ip(), verif_req.source_addr.port()),
    })
    .expect("error sending verification response to server");
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
        .to_protobuf()
        .expect("error encoding discovery request")
        .to_vec();

    server_tx
        .send(OutgoingPacket {
            msg_type: MessageType::DiscoveryRequest,
            msg_bytes: disc_req_bytes,
            target_socket_addr: target_addr,
        })
        .expect("error sending discovery request to server");
}

fn reply_with_discovery_response(
    disc_req: &DiscoveryRequest,
    msg_bytes: &[u8],
    tx: &ServerTx,
    local: &Local,
    target_addr: SocketAddr,
) {
    log::debug!("Replying with discovery response.");

    let request_hash = msg_hash(MessageType::DiscoveryRequest, msg_bytes).to_vec();

    // TODO: create an actual random set of peers.
    let peers = Vec::new();

    let disc_res = DiscoveryResponse::new(request_hash, peers);
    let disc_res_bytes = disc_res
        .to_protobuf()
        .expect("error encoding discovery response")
        .to_vec();

    tx.send(OutgoingPacket {
        msg_type: MessageType::DiscoveryResponse,
        msg_bytes: disc_res_bytes,
        target_socket_addr: target_addr,
    })
    .expect("error sending verification response to server");
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// SENDING
///////////////////////////////////////////////////////////////////////////////////////////////////////////

fn send_verification_request(target: &Peer, request_mngr: &RequestManager, server_tx: &ServerTx) {
    log::debug!("Sending verification request to: {:?}", target);

    let verif_req = request_mngr.new_verification_request(target.peer_id(), target.ip_address());
    let verif_req_bytes = verif_req
        .to_protobuf()
        .expect("error encoding verification request")
        .to_vec();

    let port = target
        .services()
        .get(AUTOPEERING_SERVICE_NAME)
        .expect("peer doesn't support autopeering")
        .port();

    server_tx
        .send(OutgoingPacket {
            msg_type: MessageType::VerificationRequest,
            msg_bytes: verif_req_bytes,
            target_socket_addr: SocketAddr::new(target.ip_address(), port),
        })
        .expect("error sending verification request to server");
}

fn send_discovery_request(target: &Peer, request_mngr: &RequestManager, server_tx: &ServerTx) {
    log::debug!("Sending discovery request to: {:?}", target);

    let disc_req = request_mngr.new_discovery_request(target.peer_id(), target.ip_address());
    let disc_req_bytes = disc_req
        .to_protobuf()
        .expect("error encoding verification request")
        .to_vec();

    let port = target
        .services()
        .get(AUTOPEERING_SERVICE_NAME)
        .expect("peer doesn't support autopeering")
        .port();

    server_tx
        .send(OutgoingPacket {
            msg_type: MessageType::DiscoveryRequest,
            msg_bytes: disc_req_bytes,
            target_socket_addr: SocketAddr::new(target.ip_address(), port),
        })
        .expect("error sending discovery request to server");
}
