// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    command::{self, Command, CommandRx, CommandTx},
    config::AutopeeringConfig,
    delay::{DelayFactory, DelayFactoryBuilder, DelayFactoryMode},
    discovery_messages::{DiscoveryRequest, DiscoveryResponse, VerificationRequest, VerificationResponse},
    event::{Event, EventTx},
    hash,
    identity::PeerId,
    local::Local,
    multiaddr::{AddressKind, AutopeeringMultiaddr},
    packet::{msg_hash, IncomingPacket, MessageType, OutgoingPacket},
    peer::{self, Peer},
    peerlist::{ActivePeersList, MasterPeersList, ReplacementList},
    peerstore::{self, PeerStore},
    request::{self, RequestManager},
    ring::PeerRing,
    server::{marshal, ServerRx, ServerSocket, ServerTx},
    service_map::{ServiceMap, ServicePort, ServiceTransport, AUTOPEERING_SERVICE_NAME},
    task::{Runnable, ShutdownBusRegistry, ShutdownRx, Task},
    time::{self, SECOND},
};

use rand::{seq::index, thread_rng, Rng as _};
use tokio::sync::mpsc;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt, iter,
    mem::replace,
    net::SocketAddr,
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock},
    time::Duration,
};

// Time interval after which the next peer is reverified.
pub(crate) const DEFAULT_REVERIFY_INTERVAL_SECS: u64 = 10 * SECOND;
// Time interval after which peers are queried for new peers.
pub(crate) const DEFAULT_QUERY_INTERVAL_SECS: u64 = 60 * SECOND;
// The default delay between requests to a single peer.
const BACKOFF_INTERVALL_MILLISECS: u64 = 500;
// A factor that determines the range from which a concrete delay is picked randomly.
const JITTER: f32 = 0.5;
// A factor that determines the intervall lengths between repeated requests to a peer.
const EXPONENTIAL_BACKOFF_FACTOR: f32 = 1.5;
// The number of times a request is repeated in case the peer doesn't reply.
const MAX_RETRIES: usize = 2;
// Is the time until a peer verification expires (12 hours).
pub(crate) const VERIFICATION_EXPIRATION_SECS: u64 = 12 * time::HOUR;
// Is the maximum number of peers returned in DiscoveryResponse.
const MAX_PEERS_IN_RESPONSE: usize = 6;
// Is the minimum number of verifications required to be selected in DiscoveryResponse.
const MIN_VERIFIED_IN_RESPONSE: usize = 1;
// Is the maximum number of services a peer can support.
const MAX_SERVICES: usize = 5;

pub(crate) struct DiscoveryManagerConfig {
    pub(crate) entry_nodes: Vec<AutopeeringMultiaddr>,
    pub(crate) entry_nodes_prefer_ipv6: bool,
    pub(crate) run_as_entry_node: bool,
    pub(crate) version: u32,
    pub(crate) network_id: u32,
    pub(crate) bind_addr: SocketAddr,
}

impl DiscoveryManagerConfig {
    pub fn new(config: &AutopeeringConfig, version: u32, network_id: u32) -> Self {
        Self {
            entry_nodes: config.entry_nodes.clone(),
            entry_nodes_prefer_ipv6: config.entry_nodes_prefer_ipv6,
            run_as_entry_node: config.run_as_entry_node,
            version,
            network_id,
            bind_addr: config.bind_addr,
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
    event_tx: EventTx,
    // The storage for discovered peers.
    peerstore: S,
    // The list of master peers.
    master_peers: MasterPeersList,
    // The list of managed peers.
    active_peers: ActivePeersList,
    // The list of replacement peers.
    replacements: ReplacementList,
}

impl<S: PeerStore + 'static> DiscoveryManager<S> {
    pub(crate) fn new(
        config: DiscoveryManagerConfig,
        local: Local,
        socket: ServerSocket,
        request_mngr: RequestManager,
        peerstore: S,
        event_tx: EventTx,
    ) -> Self {
        let mut master_peers = HashSet::with_capacity(config.entry_nodes.len());
        master_peers.extend(
            config
                .entry_nodes
                .iter()
                .map(|e| PeerId::from_public_key(e.public_key().clone())),
        );

        Self {
            config,
            local,
            socket,
            request_mngr,
            event_tx,
            peerstore,
            master_peers,
            active_peers: ActivePeersList::default(),
            replacements: ReplacementList::default(),
        }
    }

    pub async fn init(self, shutdown_reg: &mut ShutdownBusRegistry) -> CommandTx {
        let DiscoveryManager {
            config,
            local,
            socket,
            request_mngr,
            event_tx,
            peerstore,
            master_peers,
            mut active_peers,
            mut replacements,
        } = self;

        let DiscoveryManagerConfig {
            mut entry_nodes,
            entry_nodes_prefer_ipv6,
            run_as_entry_node,
            version,
            network_id,
            bind_addr,
        } = config;

        let ServerSocket {
            mut server_rx,
            server_tx,
        } = socket;

        add_entry_nodes(
            &mut entry_nodes,
            entry_nodes_prefer_ipv6,
            &request_mngr,
            &server_tx,
            &local,
            &peerstore,
            &mut active_peers,
            &mut replacements,
        )
        .await;

        add_peers_from_store(&peerstore, &local, &mut active_peers, &mut replacements).await;

        let discovery_recv_handler = DiscoveryRecvHandler {
            server_tx: server_tx.clone(),
            server_rx,
            local: local.clone(),
            peerstore: peerstore.clone(),
            version,
            network_id,
            request_mngr: request_mngr.clone(),
            event_tx,
            active_peers: active_peers.clone(),
            replacements,
        };

        let (command_tx, command_rx) = command::command_chan();

        let discovery_send_handler = DiscoverySendHandler {
            server_tx,
            local,
            peerstore,
            request_mngr,
            active_peers,
            command_rx,
        };

        Task::spawn_runnable(discovery_recv_handler, shutdown_reg.register());
        Task::spawn_runnable(discovery_send_handler, shutdown_reg.register());

        command_tx
    }
}

struct DiscoveryRecvHandler<S: PeerStore> {
    server_rx: ServerRx,
    server_tx: ServerTx,
    local: Local,
    peerstore: S,
    version: u32,
    network_id: u32,
    request_mngr: RequestManager,
    event_tx: EventTx,
    active_peers: ActivePeersList,
    replacements: ReplacementList,
}

struct DiscoverySendHandler<S: PeerStore> {
    server_tx: ServerTx,
    local: Local,
    peerstore: S,
    request_mngr: RequestManager,
    active_peers: ActivePeersList,
    command_rx: CommandRx,
}

#[async_trait::async_trait]
impl<S: PeerStore> Runnable for DiscoveryRecvHandler<S> {
    const NAME: &'static str = "DiscoveryRecvHandler";

    type Cancel = ShutdownRx;

    async fn run(self, mut shutdown_rx: Self::Cancel) {
        let DiscoveryRecvHandler {
            mut server_rx,
            server_tx,
            local,
            peerstore,
            version,
            network_id,
            request_mngr,
            event_tx,
            active_peers,
            mut replacements,
        } = self;

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
                            source_socket_addr,
                            event_tx: &event_tx,
                            active_peers: &active_peers,
                            replacements: &mut replacements,
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
                                    handle_verification_request(verif_req, ctx);
                                }
                            }
                            MessageType::VerificationResponse => {
                                let verif_res = if let Ok(verif_res) = VerificationResponse::from_protobuf(&msg_bytes) {
                                    verif_res
                                } else {
                                    log::debug!("Error decoding verification response from {}.", &peer_id);
                                    continue 'recv;
                                };

                                if let Err(e) = validate_verification_response(&verif_res, &request_mngr, &peer_id, source_socket_addr) {
                                    log::debug!("Received invalid verification response from {}. Reason: {:?}", &peer_id, e);
                                    continue 'recv;
                                } else {
                                    log::debug!("Received valid verification response from {}.", &peer_id);
                                    handle_verification_response(verif_res, ctx);
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
                                    handle_discovery_request(disc_req, ctx);
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
                                    handle_discovery_response(disc_res, ctx);
                                }
                            }
                            _ => log::debug!("Received unsupported discovery message type"),
                        }
                    }
                }
            }
        }

        // TODO: Write 'ActivePeerlist' to storage.
    }
}

#[async_trait::async_trait]
impl<S: PeerStore> Runnable for DiscoverySendHandler<S> {
    const NAME: &'static str = "DiscoverySendHandler";

    type Cancel = ShutdownRx;

    async fn run(self, mut shutdown_rx: Self::Cancel) {
        let DiscoverySendHandler {
            server_tx,
            local,
            peerstore,
            request_mngr,
            active_peers,
            mut command_rx,
        } = self;

        'recv: loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    break;
                }
                o = command_rx.recv() => {
                    if let Some(command) = o {
                        match command {
                            Command::SendVerificationRequest { peer_id } => {
                                // send_verification_request(target, request_mngr, server_tx);
                            }
                            Command::SendDiscoveryRequest { peer_id } => {
                                // send_discovery_request(target, request_mngr, server_tx);
                            }
                            Command::SendVerificationRequests => {
                                send_verification_requests(&active_peers, &peerstore, &request_mngr, &server_tx);
                            }
                            Command::SendDiscoveryRequests => {
                                send_discovery_requests(&active_peers, &peerstore, &request_mngr, &server_tx);
                            }
                        }
                    }
                }
            }
        }
    }
}

// Send regular (re-)verification requests.
pub(crate) fn send_verification_requests<S: PeerStore>(
    active_peers: &ActivePeersList,
    peerstore: &S,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
) {
    log::debug!("Sending verification request to peers.");

    for active_peer in active_peers.read_inner().iter() {
        let peer = peerstore.peer(active_peer.peer_id()).expect("inconsistent peerstore");
        let peer_id = peer.peer_id();
        let target_addr = peer.ip_address();

        let verif_req = request_mngr.new_verification_request(peer_id.clone(), target_addr);
        let msg_bytes = verif_req
            .to_protobuf()
            .expect("error encoding verification request")
            .to_vec();

        let port = peer
            .services()
            .get(AUTOPEERING_SERVICE_NAME)
            .expect("missing autopeering service")
            .port();

        server_tx.send(OutgoingPacket {
            msg_type: MessageType::VerificationRequest,
            msg_bytes,
            target_socket_addr: SocketAddr::new(target_addr, port),
        });
    }
}

// Send discovery requests to all verified peers.
pub(crate) fn send_discovery_requests<S: PeerStore>(
    active_peers: &ActivePeersList,
    peerstore: &S,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
) {
    log::debug!("Sending discovery requests to peers.");

    for active_peer in active_peers.read_inner().iter() {
        let peer = peerstore.peer(active_peer.peer_id()).expect("inconsistent peerstore");
        let peer_id = peer.peer_id();
        let target_addr = peer.ip_address();

        if peer::is_verified(peerstore.last_verification_response(peer_id)) {
            // TODO: refactor into a function
            let disc_req = request_mngr.new_discovery_request(peer_id.clone(), target_addr);
            let msg_bytes = disc_req
                .to_protobuf()
                .expect("error encoding discovery request")
                .to_vec();

            let port = peer
                .services()
                .get(AUTOPEERING_SERVICE_NAME)
                .expect("missing autopeering service")
                .port();

            server_tx.send(OutgoingPacket {
                msg_type: MessageType::DiscoveryRequest,
                msg_bytes,
                target_socket_addr: SocketAddr::new(target_addr, port),
            });
        }
    }
}

async fn add_entry_nodes<S: PeerStore>(
    entry_nodes: &mut Vec<AutopeeringMultiaddr>,
    entry_nodes_prefer_ipv6: bool,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
    local: &Local,
    peerstore: &S,
    active_peers: &mut ActivePeersList,
    replacements: &mut ReplacementList,
) {
    let mut num_added = 0;

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

        // Add peer to peer list/s.
        if add_peer_to_list(peer.peer_id().clone(), &local, active_peers, replacements) {
            num_added += 1;
        }

        // Add peer to storage.
        let _ = add_peer_to_store(peer, peerstore);
    }

    log::debug!("Added {} entry node/s.", num_added);
}

async fn add_peers_from_store<S: PeerStore>(
    peerstore: &S,
    local: &Local,
    active_peers: &ActivePeersList,
    replacements: &mut ReplacementList,
) {
    let mut num_added = 0;

    for peer in peerstore.peers() {
        if add_peer_to_list(peer.into_id(), &local, active_peers, replacements) {
            num_added += 1;
        }
    }

    log::debug!("Added {} stored peer/s.", num_added);
}

// From hive.go:
// Adds a newly discovered peer that has never been verified or pinged yet.
// It returns true, if the given peer was new and added, false otherwise.
fn add_peer_to_list(
    peer_id: PeerId,
    local: &Local,
    active_peers: &ActivePeersList,
    replacements: &mut ReplacementList,
) -> bool {
    // Do not add the local peer.
    if peer_id == local.peer_id() {
        false
    } else if active_peers.read_inner().contains(&peer_id) {
        false
    } else {
        if active_peers.read_inner().is_full() {
            replacements.append(peer_id)
        } else {
            active_peers.write_inner().append(peer_id.into())
        }
    }
}

// Writes the full peer data to the storage.
fn add_peer_to_store<S: PeerStore>(peer: Peer, peerstore: &S) -> bool {
    peerstore.insert_peer(peer)
}

fn delete_peer_from_list(
    peer_id: &PeerId,
    master_peers: &MasterPeersList,
    active_peers: &ActivePeersList,
    replacements: &mut ReplacementList,
    event_tx: EventTx,
) {
    if let Some(mut removed_peer) = active_peers.write_inner().remove(peer_id) {
        // hive.go: master peers are never removed
        if master_peers.contains(removed_peer.peer_id()) {
            //hive.go: reset verifiedCount and re-add them
            removed_peer.metrics_mut().reset_verified_count();
            active_peers.write_inner().append(removed_peer);
        } else {
            // TODO: why is the event only triggered for verified peers?
            // ```go
            // if mp.verifiedCount.Load() > 0 {
            //     m.events.PeerDeleted.Trigger(&DeletedEvent{Peer: unwrapPeer(mp)})
            // }
            // ```
            if removed_peer.metrics().verified_count() > 0 {
                event_tx
                    .send(Event::PeerDeleted {
                        peer_id: peer_id.clone(),
                    })
                    .expect("error sending `PeerDeleted` event");
            }

            // ```go
            // // add a random replacement, if available
            // if len(m.replacements) > 0 {
            // 	var r *mpeer
            // 	m.replacements, r = deletePeer(m.replacements, rand.Intn(len(m.replacements)))
            // 	m.active = pushPeer(m.active, r, maxManaged)
            // }
            // ```
            if !replacements.is_empty() {
                let index = rand::thread_rng().gen_range(0..replacements.len());
                // Panic: unwrapping is fine, because we checked that the list isn't empty, and `index` must be in range.
                let r = replacements.remove_at(index).unwrap();
                active_peers.write_inner().append(r.into());
            }
        }
    }
}

// Determines whether the peer id is known.
fn is_known_peer_id(
    peer_id: &PeerId,
    local: &Local,
    active_peers: &ActivePeersList,
    replacements: &ReplacementList,
) -> bool {
    // Note: master peers are always a subset of the active peers.
    peer_id == &local.peer_id() || active_peers.read_inner().contains(&peer_id) || replacements.contains(peer_id)
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
    NoCorrespondingRequestOrTimeout,
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
    source_socket_addr: SocketAddr,
) -> Result<(), ValidationError> {
    use ValidationError::*;

    if let Some(request_hash) = request_mngr.get_request_hash::<VerificationRequest>(peer_id) {
        if verif_res.request_hash() == &request_hash[..] {
            let res_services = verif_res.services();
            if let Some(res_peering) = res_services.get(AUTOPEERING_SERVICE_NAME) {
                if res_peering.port() == source_socket_addr.port() {
                    Ok(())
                } else {
                    Err(ServicePortMismatch {
                        expected: source_socket_addr.port(),
                        received: res_peering.port(),
                    })
                }
            } else {
                Err(NoAutopeeringService)
            }
        } else {
            Err(IncorrectRequestHash)
        }
    } else {
        Err(NoCorrespondingRequestOrTimeout)
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
        Err(NoCorrespondingRequestOrTimeout)
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
    source_socket_addr: SocketAddr,
    event_tx: &'a EventTx,
    active_peers: &'a ActivePeersList,
    replacements: &'a ReplacementList,
}

fn handle_verification_request<S: PeerStore>(verif_req: VerificationRequest, ctx: HandlerContext<S>) {
    log::debug!("Handling verification request.");

    ctx.peerstore.update_last_verification_request(ctx.peer_id.clone());

    reply_with_verification_response(
        &verif_req,
        ctx.msg_bytes,
        ctx.server_tx,
        ctx.local,
        ctx.source_socket_addr,
    );

    // ```go
    // if the peer is unknown or expired, send a Ping to verify
    // if !p.IsVerified(from.ID(), dstAddr.IP) {
    //     p.sendPing(dstAddr, from.ID())
    // } else if !p.mgr.isKnown(from.ID()) {
    //     // add a discovered peer to the manager if it is new but verified
    // 	   p.mgr.addDiscoveredPeer(newPeer(from, s.LocalAddr().Network(), dstAddr))
    // }
    // ```

    if !peer::is_verified(ctx.peerstore.last_verification_response(ctx.peer_id)) {
        reply_with_verification_request(ctx.peer_id, ctx.request_mngr, ctx.server_tx, ctx.source_socket_addr);
    }
}

fn handle_verification_response<S: PeerStore>(verif_res: VerificationResponse, ctx: HandlerContext<S>) {
    // Remove the corresponding request from the request manager.
    if ctx.request_mngr.remove_request::<VerificationRequest>(ctx.peer_id) {
        log::debug!("Handling verification response.");

        // Update verification timestamp.
        ctx.peerstore.update_last_verification_response(ctx.peer_id.clone());
    } else {
        // Either the peer sent a response already, or the response came in too late.
        log::debug!("Ignoring verification response.");
    }
}

fn handle_discovery_request<S: PeerStore>(disc_req: DiscoveryRequest, ctx: HandlerContext<S>) {
    log::debug!("Handling discovery request.");

    let request_hash = msg_hash(MessageType::DiscoveryRequest, ctx.msg_bytes).to_vec();

    // TODO: prevent the `PeerId` clone in this method, and instead pass a closure that pulls the corresponding peers from the storage.
    let chosen_peers = choose_n_random_active_peers(ctx.active_peers, MAX_PEERS_IN_RESPONSE, MIN_VERIFIED_IN_RESPONSE);

    let mut peers = Vec::with_capacity(MAX_PEERS_IN_RESPONSE);
    peers.extend(
        chosen_peers
            .into_iter()
            .filter_map(|peer_id| ctx.peerstore.peer(&peer_id)),
    );

    let disc_res = DiscoveryResponse::new(request_hash, peers);
    let disc_res_bytes = disc_res
        .to_protobuf()
        .expect("error encoding discovery response")
        .to_vec();

    ctx.server_tx
        .send(OutgoingPacket {
            msg_type: MessageType::DiscoveryResponse,
            msg_bytes: disc_res_bytes,
            target_socket_addr: ctx.source_socket_addr,
        })
        .expect("error sending verification response to server");
}

fn handle_discovery_response<S: PeerStore>(disc_res: DiscoveryResponse, ctx: HandlerContext<S>) {
    // Remove the corresponding request from the request manager.
    if ctx.request_mngr.remove_request::<DiscoveryRequest>(ctx.peer_id) {
        log::debug!("Handling discovery response.");

        // TODO: store the discovered peers; fire `PeerDiscovered` event.
        for peer in disc_res.into_peers() {
            log::debug!("{:?}", peer);

            // TODO: should the peerstore itself make the check?
            if peer.peer_id() != &ctx.local.peer_id() {
                ctx.peerstore.insert_peer(peer);
            }
        }
    } else {
        // Either the peer sent a response already, or the response came in too late.
        log::debug!("Ignoring discovery response.");
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

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// SENDING
///////////////////////////////////////////////////////////////////////////////////////////////////////////

fn send_verification_request(target: &Peer, request_mngr: &RequestManager, server_tx: &ServerTx) {
    log::debug!("Sending verification request to: {:?}", target);

    let verif_req = request_mngr.new_verification_request(target.peer_id().clone(), target.ip_address());
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

    let disc_req = request_mngr.new_discovery_request(target.peer_id().clone(), target.ip_address());
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

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// HELPERS
///////////////////////////////////////////////////////////////////////////////////////////////////////////

fn choose_n_random_active_peers(
    active_peers: &ActivePeersList,
    mut n: usize,
    min_verified_count: usize,
) -> Vec<PeerId> {
    let len = active_peers.read_inner().len();

    if active_peers.read_inner().len() <= n {
        // No randomization required => return all we got - if possible.
        let mut all_peers = Vec::with_capacity(len);
        all_peers.extend(active_peers.read_inner().iter().filter_map(|entry| {
            if entry.metrics().verified_count() >= min_verified_count {
                Some(entry.peer_id().clone())
            } else {
                None
            }
        }));
        all_peers
    } else {
        // TODO: should this better be a `CryptoRng`?
        let mut random_peers = Vec::with_capacity(n);
        let mut rng = rand::thread_rng();
        let index_vec = index::sample(&mut rng, len, len);
        random_peers.extend(
            index_vec
                .iter()
                // Panic: unwrapping is safe due to the length check.
                .map(|index| active_peers.read_inner().get(index).unwrap().clone())
                .filter_map(|entry| {
                    if entry.metrics().verified_count() >= min_verified_count {
                        Some(entry.peer_id().clone())
                    } else {
                        None
                    }
                })
                .take(n),
        );
        random_peers
    }
}
