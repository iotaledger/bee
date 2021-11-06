// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    command::{self, Command, CommandRx, CommandTx},
    config::AutopeeringConfig,
    discovery::messages::{DiscoveryRequest, DiscoveryResponse, VerificationRequest, VerificationResponse},
    event::{Event, EventTx},
    local::{
        services::{ServiceMap, ServicePort, ServiceProtocol, AUTOPEERING_SERVICE_NAME},
        Local,
    },
    multiaddr::{AddressKind, AutopeeringMultiaddr},
    packet::{msg_hash, IncomingPacket, MessageType, OutgoingPacket},
    peer::{
        self,
        peer_id::PeerId,
        peerlist::{ActivePeer, ActivePeersList, MasterPeersList, ReplacementList},
        peerstore::PeerStore,
        Peer,
    },
    request::{self, RequestManager, RequestValue, ResponseTx, RESPONSE_TIMEOUT},
    server::{ServerRx, ServerSocket, ServerTx},
    task::{Runnable, ShutdownRx, TaskManager},
    time::{self, SECOND},
};

use rand::{seq::index, Rng as _};

use std::net::SocketAddr;

// Time interval after which the next peer is reverified.
pub(crate) const DEFAULT_REVERIFY_INTERVAL_SECS: u64 = 10 * SECOND;
// Time interval after which peers are queried for new peers.
pub(crate) const DEFAULT_QUERY_INTERVAL_SECS: u64 = 60 * SECOND;
// The default delay between requests to a single peer.
// TODO: revisit dead code
#[allow(dead_code)]
const BACKOFF_INTERVALL_MILLISECS: u64 = 500;
// A factor that determines the range from which a concrete delay is picked randomly.
// TODO: revisit dead code
#[allow(dead_code)]
const JITTER: f32 = 0.5;
// A factor that determines the intervall lengths between repeated requests to a peer.
// TODO: revisit dead code
#[allow(dead_code)]
const EXPONENTIAL_BACKOFF_FACTOR: f32 = 1.5;
// The number of times a request is repeated in case the peer doesn't reply.
// TODO: revisit dead code
#[allow(dead_code)]
const MAX_RETRIES: usize = 2;
// Is the time until a peer verification expires (12 hours).
pub(crate) const VERIFICATION_EXPIRATION_SECS: u64 = 12 * time::HOUR;
// Is the maximum number of peers returned in DiscoveryResponse.
const MAX_PEERS_IN_RESPONSE: usize = 6;
// Is the minimum number of verifications required to be selected in DiscoveryResponse.
const MIN_VERIFIED_IN_RESPONSE: usize = 1;
// Is the maximum number of services a peer can support.
// TODO: revisit dead code
#[allow(dead_code)]
const MAX_SERVICES: usize = 5;

// pub(crate) type DiscoveryHandler<S: PeerStore> = Box<dyn Fn(&RecvContext<S>) + Send + Sync + 'static>;

pub(crate) struct DiscoveryManagerConfig {
    pub(crate) entry_nodes: Vec<AutopeeringMultiaddr>,
    pub(crate) entry_nodes_prefer_ipv6: bool,
    // TODO: revisit dead code
    #[allow(dead_code)]
    pub(crate) run_as_entry_node: bool,
    pub(crate) version: u32,
    pub(crate) network_id: u32,
    // TODO: revisit dead code
    #[allow(dead_code)]
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

pub(crate) struct DiscoveryManager<S: PeerStore> {
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
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        config: DiscoveryManagerConfig,
        local: Local,
        socket: ServerSocket,
        request_mngr: RequestManager,
        peerstore: S,
        master_peers: MasterPeersList,
        active_peers: ActivePeersList,
        replacements: ReplacementList,
        event_tx: EventTx,
    ) -> Self {
        Self {
            config,
            local,
            socket,
            request_mngr,
            event_tx,
            peerstore,
            master_peers,
            active_peers,
            replacements,
        }
    }

    pub async fn init<const N: usize>(self, task_mngr: &mut TaskManager<S, N>) -> CommandTx {
        let DiscoveryManager {
            config,
            local,
            socket,
            request_mngr,
            event_tx,
            peerstore,
            master_peers,
            active_peers,
            replacements,
        } = self;

        let DiscoveryManagerConfig {
            mut entry_nodes,
            entry_nodes_prefer_ipv6,
            run_as_entry_node: _,
            version,
            network_id,
            bind_addr: _,
        } = config;

        let ServerSocket { server_rx, server_tx } = socket;

        // Add previously discovered peers from the peer store.
        add_peers_from_store(&peerstore, &active_peers, &replacements);

        // Add master peers from the config.
        add_master_peers(
            &mut entry_nodes,
            entry_nodes_prefer_ipv6,
            &local,
            &master_peers,
            &active_peers,
            &replacements,
        )
        .await;

        let discovery_recv_handler = DiscoveryRecvHandler {
            server_tx: server_tx.clone(),
            server_rx,
            local: local.clone(),
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
            request_mngr,
            active_peers,
            command_rx,
        };

        task_mngr.run::<DiscoveryRecvHandler>(discovery_recv_handler);
        task_mngr.run::<DiscoverySendHandler>(discovery_send_handler);

        command_tx
    }
}

struct DiscoveryRecvHandler {
    server_rx: ServerRx,
    server_tx: ServerTx,
    local: Local,
    version: u32,
    network_id: u32,
    request_mngr: RequestManager,
    event_tx: EventTx,
    active_peers: ActivePeersList,
    replacements: ReplacementList,
}

struct DiscoverySendHandler {
    server_tx: ServerTx,
    request_mngr: RequestManager,
    active_peers: ActivePeersList,
    command_rx: CommandRx,
}

#[async_trait::async_trait]
impl Runnable for DiscoveryRecvHandler {
    const NAME: &'static str = "DiscoveryRecvHandler";
    const SHUTDOWN_PRIORITY: u8 = 4;

    type ShutdownSignal = ShutdownRx;

    async fn run(self, mut shutdown_rx: Self::ShutdownSignal) {
        let DiscoveryRecvHandler {
            mut server_rx,
            server_tx,
            local,
            version,
            network_id,
            request_mngr,
            event_tx,
            active_peers,
            replacements,
            ..
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
                        peer_addr,
                        peer_id,
                    }) = o
                    {
                        let ctx = RecvContext {
                            peer_id: &peer_id,
                            msg_bytes: &msg_bytes,
                            server_tx: &server_tx,
                            local: &local,
                            request_mngr: &request_mngr,
                            peer_addr,
                            event_tx: &event_tx,
                            active_peers: &active_peers,
                            replacements: &replacements,
                        };

                        match msg_type {
                            MessageType::VerificationRequest => {
                                let verif_req = if let Ok(verif_req) = VerificationRequest::from_protobuf(&msg_bytes) {
                                    verif_req
                                } else {
                                    log::warn!("Error decoding verification request from {}.", &peer_id);
                                    continue 'recv;
                                };

                                if let Err(e) = validate_verification_request(&verif_req, version, network_id) {
                                    log::warn!("Received invalid verification request from {}. Reason: {:?}", &peer_id, e);
                                    continue 'recv;
                                } else {
                                    log::trace!("Received valid verification request from {}.", &peer_id);

                                    handle_verification_request(verif_req, ctx);
                                }
                            }
                            MessageType::VerificationResponse => {
                                let verif_res = if let Ok(verif_res) = VerificationResponse::from_protobuf(&msg_bytes) {
                                    verif_res
                                } else {
                                    log::warn!("Error decoding verification response from {}.", &peer_id);
                                    continue 'recv;
                                };

                                match validate_verification_response(&verif_res, &request_mngr, &peer_id, peer_addr) {
                                    Ok(verif_reqval) => {
                                        log::trace!("Received valid verification response from {}.", &peer_id);

                                        handle_verification_response(verif_res, verif_reqval, ctx);
                                    }
                                    Err(e) => {
                                        log::warn!("Received invalid verification response from {}. Reason: {:?}", &peer_id, e);
                                        continue 'recv;
                                    }
                                }
                            }
                            MessageType::DiscoveryRequest => {
                                let disc_req = if let Ok(disc_req) = DiscoveryRequest::from_protobuf(&msg_bytes) {
                                    disc_req
                                } else {
                                    log::warn!("Error decoding discovery request from {}.", &peer_id);
                                    continue 'recv;
                                };

                                if let Err(e) = validate_discovery_request(&disc_req) {
                                    log::warn!("Received invalid discovery request from {}. Reason: {:?}", &peer_id, e);
                                    continue 'recv;
                                } else {
                                    log::trace!("Received valid discovery request from {}.", &peer_id);

                                    handle_discovery_request(disc_req, ctx);
                                }
                            }
                            MessageType::DiscoveryResponse => {
                                let disc_res = if let Ok(disc_res) = DiscoveryResponse::from_protobuf(&msg_bytes) {
                                    disc_res
                                } else {
                                    log::warn!("Error decoding discovery response from {}.", &peer_id);
                                    continue 'recv;
                                };

                                match validate_discovery_response(&disc_res, &request_mngr, &peer_id) {
                                    Ok(disc_reqval) => {
                                        log::trace!("Received valid discovery response from {}.", &peer_id);

                                        handle_discovery_response(disc_res, disc_reqval, ctx);
                                    }
                                    Err(e) => {
                                        log::warn!("Received invalid discovery response from {}. Reason: {:?}", &peer_id, e);
                                        continue 'recv;
                                    }
                                }
                            }
                            _ => log::warn!("Received unsupported discovery message type"),
                        }
                    }
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl Runnable for DiscoverySendHandler {
    const NAME: &'static str = "DiscoverySendHandler";
    const SHUTDOWN_PRIORITY: u8 = 5;

    type ShutdownSignal = ShutdownRx;

    async fn run(self, mut shutdown_rx: Self::ShutdownSignal) {
        let DiscoverySendHandler {
            server_tx,
            request_mngr,
            active_peers,
            mut command_rx,
        } = self;

        'recv: loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    break 'recv;
                }
                o = command_rx.recv() => {
                    if let Some(command) = o {
                        match command {
                            Command::Verify{ peer_id } => {
                                send_verification_request_to_peer(&peer_id, &active_peers, &request_mngr, &server_tx, None);
                            }
                            Command::Query{ peer_id } => {
                                send_discovery_request_to_peer(&peer_id, &active_peers, &request_mngr, &server_tx, None);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

fn add_peers_from_store<S: PeerStore>(peerstore: &S, active_peers: &ActivePeersList, replacements: &ReplacementList) {
    let mut num_added = 0;

    let mut write = active_peers.write();
    for active_peer in peerstore.fetch_all_active() {
        write.insert(active_peer);
        num_added += 1;
    }
    drop(write);
    let mut write = replacements.write();
    for replacement in peerstore.fetch_all_replacements() {
        write.insert(replacement);
        num_added += 1;
    }
    drop(write);

    log::debug!("Restored {} peer/s.", num_added);
}

async fn add_master_peers(
    entry_nodes: &mut Vec<AutopeeringMultiaddr>,
    entry_nodes_prefer_ipv6: bool,
    local: &Local,
    master_peers: &MasterPeersList,
    active_peers: &ActivePeersList,
    replacements: &ReplacementList,
) {
    let mut num_added = 0;

    for entry_addr in entry_nodes {
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

        let mut peer = Peer::new(entry_socketaddr.ip(), *entry_addr.public_key());
        peer.add_service(AUTOPEERING_SERVICE_NAME, ServiceProtocol::Udp, entry_socketaddr.port());

        master_peers.write().insert(*peer.peer_id());

        // Also add it as a regular peer.
        if let Some(peer_id) = add_peer::<false>(peer, local, active_peers, replacements) {
            log::debug!("Added {}.", peer_id);
            num_added += 1;
        }
    }

    log::debug!("Added {} entry node/s.", num_added);
}

/// Attempts to add a new peer to a peer list (preferably as active).
/// If the peer is added inbound, i.e. , the "last verification timestamp" is added.
pub(crate) fn add_peer<const ON_REQUEST: bool>(
    peer: Peer,
    local: &Local,
    active_peers: &ActivePeersList,
    replacements: &ReplacementList,
) -> Option<PeerId> {
    // Only add new peers.
    if peer::is_known(peer.peer_id(), local, active_peers, replacements) {
        None
    } else {
        let peer_id = *peer.peer_id();
        // First try to add it to the active peer list. If that list is full, add it to the replacement list.
        if !active_peers.read().is_full() {
            let active_peer = if ON_REQUEST {
                let mut active = ActivePeer::from(peer);
                active.metrics_mut().set_last_verif_request_timestamp();
                active
            } else {
                ActivePeer::from(peer)
            };
            if active_peers.write().insert(active_peer) {
                Some(peer_id)
            } else {
                None
            }
        } else if replacements.write().insert(peer) {
            Some(peer_id)
        } else {
            None
        }
    }
}

// Note: this function is dead-lock danger zone!
/// Deletes a peer from the active peerlist if it's not a master peer, and replaces it by a peer
/// from the replacement list.
pub(crate) fn remove_peer_from_active_list(
    peer_id: &PeerId,
    master_peers: &MasterPeersList,
    active_peers: &ActivePeersList,
    replacements: &ReplacementList,
    event_tx: &EventTx,
) {
    let mut active_peers = active_peers.write();

    if let Some(mut removed_peer) = active_peers.remove(peer_id) {
        // hive.go: master peers are never removed
        if master_peers.read().contains(removed_peer.peer_id()) {
            // hive.go: reset verifiedCount and re-add them
            removed_peer.metrics_mut().reset_verified_count();
            active_peers.insert(removed_peer);
        } else {
            // TODO: why is the event only triggered for verified peers?
            // ```go
            // if mp.verifiedCount.Load() > 0 {
            //     m.events.PeerDeleted.Trigger(&DeletedEvent{Peer: unwrapPeer(mp)})
            // }
            // ```
            if removed_peer.metrics().verified_count() > 0 {
                event_tx
                    .send(Event::PeerDeleted { peer_id: *peer_id })
                    .expect("error sending `PeerDeleted` event");
            }

            // ```go
            // if len(m.replacements) > 0 {
            // 	var r *mpeer
            // 	m.replacements, r = deletePeer(m.replacements, rand.Intn(len(m.replacements)))
            // 	m.active = pushPeer(m.active, r, maxManaged)
            // }
            // ```
            // Pick a random peer from the replacement list (if not empty)
            if !replacements.read().is_empty() {
                let index = rand::thread_rng().gen_range(0..replacements.read().len());
                // Panic: unwrapping is fine, because we checked that the list isn't empty, and `index` must be in
                // range.
                let peer = replacements.write().remove_at(index).unwrap();

                active_peers.insert(peer.into());
            }
        }
    }
}

pub(crate) struct RecvContext<'a> {
    peer_id: &'a PeerId,
    msg_bytes: &'a [u8],
    server_tx: &'a ServerTx,
    local: &'a Local,
    request_mngr: &'a RequestManager,
    peer_addr: SocketAddr,
    event_tx: &'a EventTx,
    active_peers: &'a ActivePeersList,
    replacements: &'a ReplacementList,
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// VALIDATION
///////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub(crate) enum ValidationError {
    // The protocol version must match.
    // TODO: revisit dead code
    #[allow(dead_code)]
    VersionMismatch {
        expected: u32,
        received: u32,
    },
    // The network id must match.
    // TODO: revisit dead code
    #[allow(dead_code)]
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
    // TODO: revisit dead code
    #[allow(dead_code)]
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
) -> Result<RequestValue, ValidationError> {
    use ValidationError::*;

    if let Some(reqv) = request_mngr.write().pull::<VerificationRequest>(peer_id) {
        if verif_res.request_hash() == reqv.request_hash {
            let res_services = verif_res.services();
            if let Some(autopeering_svc) = res_services.get(AUTOPEERING_SERVICE_NAME) {
                if autopeering_svc.port() == source_socket_addr.port() {
                    Ok(reqv)
                } else {
                    Err(ServicePortMismatch {
                        expected: source_socket_addr.port(),
                        received: autopeering_svc.port(),
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
) -> Result<RequestValue, ValidationError> {
    use ValidationError::*;

    if let Some(reqv) = request_mngr.write().pull::<DiscoveryRequest>(peer_id) {
        if disc_res.request_hash() == &reqv.request_hash[..] {
            // TODO: consider performing some checks on the peers we received, for example:
            // * does the peer have necessary services (autopeering, gossip, fpc, ...)
            // * is the ip address valid (not a 0.0.0.0, etc)
            // for peer in disc_res.peers() {}

            Ok(reqv)
        } else {
            Err(IncorrectRequestHash)
        }
    } else {
        Err(NoCorrespondingRequestOrTimeout)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// HANDLING
///////////////////////////////////////////////////////////////////////////////////////////////////////////

fn handle_verification_request(verif_req: VerificationRequest, ctx: RecvContext) {
    log::trace!("Handling verification request.");

    // In any case send a response.
    send_verification_response_to_addr(
        ctx.peer_addr,
        ctx.peer_id,
        &verif_req,
        ctx.msg_bytes,
        ctx.server_tx,
        ctx.local,
    );

    // Is this a known peer?
    if peer::is_known(ctx.peer_id, ctx.local, ctx.active_peers, ctx.replacements) {
        // Update verification request timestamp
        if let Some(peer) = ctx.active_peers.write().find_mut(ctx.peer_id) {
            peer.metrics_mut().set_last_verif_request_timestamp();
        }

        if !peer::is_verified(ctx.peer_id, ctx.active_peers) {
            // Peer is known, but no longer verified.
            send_verification_request_to_addr(ctx.peer_addr, ctx.peer_id, ctx.request_mngr, ctx.server_tx, None);
        }
    } else {
        // Add it as a new peer with autopeering service.
        let mut peer = Peer::new(ctx.peer_addr.ip(), *ctx.peer_id.public_key());
        peer.add_service(AUTOPEERING_SERVICE_NAME, ServiceProtocol::Udp, ctx.peer_addr.port());

        if let Some(peer_id) = add_peer::<true>(peer, ctx.local, ctx.active_peers, ctx.replacements) {
            log::debug!("Added {}.", peer_id);
        }

        // Peer is unknown, thus still unverified.
        send_verification_request_to_addr(ctx.peer_addr, ctx.peer_id, ctx.request_mngr, ctx.server_tx, None);
    }
}

// The peer must be known (since it's a valid response). That means that the peer is part of the active list currently.
fn handle_verification_response(verif_res: VerificationResponse, verif_reqval: RequestValue, ctx: RecvContext) {
    log::trace!("Handling verification response.");

    if let Some(verified_count) = peer::set_front_and_update(ctx.peer_id, ctx.active_peers) {
        // If this is the first time the peer was verified:
        // * Update its services;
        // * Fire the "peer discovered" event;
        if verified_count == 1 {
            if let Some(peer) = ctx.active_peers.write().find_mut(ctx.peer_id) {
                peer.peer_mut().set_services(verif_res.services().clone())
            }

            ctx.event_tx
                .send(Event::PeerDiscovered { peer_id: *ctx.peer_id })
                .expect("error publishing peer-discovered event");
        }
    }

    // Send the response notification.
    if let Some(tx) = verif_reqval.response_tx {
        tx.send(
            verif_res
                .to_protobuf()
                .expect("error encoding discovery response")
                .to_vec(),
        )
        .expect("error sending response signal");
    }
}

fn handle_discovery_request(_disc_req: DiscoveryRequest, ctx: RecvContext) {
    log::trace!("Handling discovery request.");

    let request_hash = msg_hash(MessageType::DiscoveryRequest, ctx.msg_bytes).to_vec();

    let chosen_peers =
        choose_n_random_peers_from_active_list(ctx.active_peers, MAX_PEERS_IN_RESPONSE, MIN_VERIFIED_IN_RESPONSE);

    let disc_res = DiscoveryResponse::new(request_hash, chosen_peers);
    let disc_res_bytes = disc_res
        .to_protobuf()
        .expect("error encoding discovery response")
        .to_vec();

    ctx.server_tx
        .send(OutgoingPacket {
            msg_type: MessageType::DiscoveryResponse,
            msg_bytes: disc_res_bytes,
            peer_addr: ctx.peer_addr,
        })
        .expect("error sending verification response to server");
}

fn handle_discovery_response(disc_res: DiscoveryResponse, disc_reqval: RequestValue, ctx: RecvContext) {
    // Remove the corresponding request from the request manager.
    log::trace!("Handling discovery response.");

    let mut num_added = 0;

    // Add discovered peers to the peer list and peer store.
    for peer in disc_res.into_peers() {
        // Note: we only fire `PeerDiscovered` if it can be verified.
        if let Some(peer_id) = add_peer::<false>(peer, ctx.local, ctx.active_peers, ctx.replacements) {
            log::debug!("Added: {}.", peer_id);
            num_added += 1;
        }
    }

    // Remember how many new peers were discovered thanks to that peer.
    ctx.active_peers
        .write()
        .find_mut(ctx.peer_id)
        .expect("inconsistent active peers list")
        .metrics_mut()
        .set_last_new_peers(num_added);

    // Send the response notification.
    if let Some(tx) = disc_reqval.response_tx {
        tx.send(ctx.msg_bytes.to_vec()).expect("error sending response signal");
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// SENDING
///////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Initiates a verification request to a peer waiting for the peer's response, which must arrive in time.
///
/// Returns `Some(ServiceMap)` if the request was successful, otherwise `None`.
pub(crate) async fn begin_verification(
    peer_id: &PeerId,
    active_peers: &ActivePeersList,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
) -> Option<ServiceMap> {
    let (response_tx, response_rx) = request::response_chan();

    send_verification_request_to_peer(peer_id, active_peers, request_mngr, server_tx, Some(response_tx));

    match tokio::time::timeout(RESPONSE_TIMEOUT, response_rx).await {
        Ok(Ok(bytes)) => {
            let verif_res = VerificationResponse::from_protobuf(&bytes).expect("error decoding verification response");
            Some(verif_res.into_services())
        }
        Ok(Err(e)) => {
            log::debug!("Verification response error: {}", e);
            None
        }
        Err(e) => {
            log::debug!("Verification response timeout: {}", e);
            None
        }
    }
}

// TODO: revisit dead code
// TEMP: for testing purposes
#[allow(dead_code)]
pub(crate) fn send_verification_request_to_all_active(
    active_peers: &ActivePeersList,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
) {
    log::trace!("Sending verification request to all peers.");

    // Technically it's not necessary, but we create copies of all peer ids in order to release the lock quickly, and
    // not keep it for the whole loop.
    active_peers
        .read()
        .iter()
        .map(|p| p.peer_id())
        .copied()
        .for_each(|peer_id| {
            send_verification_request_to_peer(&peer_id, active_peers, request_mngr, server_tx, None);
        });
}

/// Sends a verification request to a peer by fetching its endpoint data from the peer store.
///
/// The function is non-blocking.
pub(crate) fn send_verification_request_to_peer(
    peer_id: &PeerId,
    active_peers: &ActivePeersList,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
    response_tx: Option<ResponseTx>,
) {
    // TEMP: guard and drop
    let guard = active_peers.read();
    if let Some(active_peer) = active_peers.read().find(peer_id) {
        drop(guard);

        let peer_port = active_peer
            .peer()
            .services()
            .get(AUTOPEERING_SERVICE_NAME)
            .expect("peer doesn't support autopeering")
            .port();

        let peer_addr = SocketAddr::new(active_peer.peer().ip_address(), peer_port);

        send_verification_request_to_addr(peer_addr, active_peer.peer_id(), request_mngr, server_tx, response_tx);
    }
}

/// Sends a verification request to a peer's address.
pub(crate) fn send_verification_request_to_addr(
    peer_addr: SocketAddr,
    peer_id: &PeerId,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
    response_tx: Option<ResponseTx>,
) {
    log::trace!("Sending verification request to: {}/{}", peer_id, peer_addr);

    let verif_req = request_mngr
        .write()
        .new_verification_request(*peer_id, peer_addr.ip(), response_tx);

    let msg_bytes = verif_req
        .to_protobuf()
        .expect("error encoding verification request")
        .to_vec();

    server_tx
        .send(OutgoingPacket {
            msg_type: MessageType::VerificationRequest,
            msg_bytes,
            peer_addr,
        })
        .expect("error sending verification request to server");
}

/// Sends a verification response to a peer's address.
pub(crate) fn send_verification_response_to_addr(
    peer_addr: SocketAddr,
    peer_id: &PeerId,
    verif_req: &VerificationRequest,
    msg_bytes: &[u8],
    server_tx: &ServerTx,
    local: &Local,
) {
    log::trace!("Sending verification response to: {}/{}", peer_id, peer_addr);

    let request_hash = msg_hash(MessageType::VerificationRequest, msg_bytes).to_vec();

    let verif_res = VerificationResponse::new(request_hash, local.read().services().clone(), peer_addr.ip());

    let msg_bytes = verif_res
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
    server_tx
        .send(OutgoingPacket {
            msg_type: MessageType::VerificationResponse,
            msg_bytes,
            peer_addr: SocketAddr::new(peer_addr.ip(), verif_req.source_addr().port()),
        })
        .expect("error sending verification response to server");
}

/// Initiates a discovery request to a peer by fetching its endpoint data from the peer store and waiting
/// for the peer's response, which must arrive in time.
///
/// Returns `Some(Vec<Peer>)` of discovered peers, if the request was successful, otherwise `None`.
pub(crate) async fn begin_discovery(
    peer_id: &PeerId,
    active_peers: &ActivePeersList,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
) -> Option<Vec<Peer>> {
    let (response_tx, response_rx) = request::response_chan();

    send_discovery_request_to_peer(peer_id, active_peers, request_mngr, server_tx, Some(response_tx));

    match tokio::time::timeout(RESPONSE_TIMEOUT, response_rx).await {
        Ok(Ok(bytes)) => {
            let disc_res = DiscoveryResponse::from_protobuf(&bytes).expect("error decoding discovery response");
            Some(disc_res.into_peers())
        }
        Ok(Err(e)) => {
            // This shouldn't happen under normal circumstances.
            log::debug!("Discovery response error: {}", e);
            Some(Vec::new())
        }
        Err(e) => {
            log::debug!("Discovery response timeout: {}", e);
            None
        }
    }
}

/// Sends a discovery request to a peer by fetching its endpoint data from the peer store.
///
/// The function is non-blocking.
pub(crate) fn send_discovery_request_to_peer(
    peer_id: &PeerId,
    active_peers: &ActivePeersList,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
    response_tx: Option<ResponseTx>,
) {
    let guard = active_peers.read();
    let active_peer = guard.find(peer_id).expect("peer not in active peers list");

    let peer_port = active_peer
        .peer()
        .services()
        .get(AUTOPEERING_SERVICE_NAME)
        .expect("peer doesn't support autopeering")
        .port();

    let peer_addr = SocketAddr::new(active_peer.peer().ip_address(), peer_port);

    send_discovery_request_to_addr(peer_addr, peer_id, request_mngr, server_tx, response_tx);
}

/// Sends a discovery request to a peer's address.
pub(crate) fn send_discovery_request_to_addr(
    peer_addr: SocketAddr,
    peer_id: &PeerId,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
    response_tx: Option<ResponseTx>,
) {
    log::trace!("Sending discovery request to: {:?}", peer_id);

    let disc_req = request_mngr.write().new_discovery_request(*peer_id, response_tx);

    let msg_bytes = disc_req
        .to_protobuf()
        .expect("error encoding verification request")
        .to_vec();

    server_tx
        .send(OutgoingPacket {
            msg_type: MessageType::DiscoveryRequest,
            msg_bytes,
            peer_addr,
        })
        .expect("error sending discovery request to server");
}

// TODO: revisit dead code
// TEMP: for testing purposes
#[allow(dead_code)]
pub(crate) fn send_discovery_request_to_all_verified(
    active_peers: &ActivePeersList,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
) {
    log::trace!("Sending discovery request to all verified peers.");

    // Technically it's not necessary, but we create copies of all peer ids in order to release the lock quickly, and
    // not keep it for the whole loop.
    let peers = active_peers
        .read()
        .iter()
        .map(|p| p.peer_id())
        .cloned()
        .collect::<Vec<_>>();

    for peer_id in &peers {
        if peer::is_verified(peer_id, active_peers) {
            send_discovery_request_to_peer(peer_id, active_peers, request_mngr, server_tx, None);
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// HELPERS
///////////////////////////////////////////////////////////////////////////////////////////////////////////

fn choose_n_random_peers_from_active_list(
    active_peers: &ActivePeersList,
    n: usize,
    min_verified_count: usize,
) -> Vec<Peer> {
    let len = active_peers.read().len();

    if active_peers.read().len() <= n {
        // No randomization required => return all we got - if possible.
        let mut all_peers = Vec::with_capacity(len);
        all_peers.extend(active_peers.read().iter().filter_map(|active| {
            if active.metrics().verified_count() >= min_verified_count {
                Some(active.peer().clone())
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
                .map(|index| active_peers.read().get(index).unwrap().clone())
                .filter_map(|active| {
                    if active.metrics().verified_count() >= min_verified_count {
                        Some(active.peer().clone())
                    } else {
                        None
                    }
                })
                .take(n),
        );
        random_peers
    }
}

// Hive.go: returns the verified peer with the given ID, or nil if no such peer exists.
// ---
// TODO: revisit dead code
// ---
/// Returns `Some(Peer)`, if either the corresponding peer is still verified, or - if it isn't -
/// can be successfully reverified.
///
/// Note: The function is blocking.
#[allow(dead_code)]
pub(crate) async fn get_verified_peer(
    peer_id: &PeerId,
    active_peers: &ActivePeersList,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
) -> Option<Peer> {
    if let Some(active_peer) = active_peers.read().find(peer_id).cloned() {
        if active_peer.metrics().verified_count() > 0 {
            return Some(active_peer.peer().clone());
        }
    } else {
        return None;
    };

    // Enforce re/verification.
    begin_verification(peer_id, active_peers, request_mngr, server_tx)
        .await
        .map(|_services| {
            active_peers
                .read()
                .find(peer_id)
                .expect("inconsistent peer list")
                .peer()
                .clone()
        })
}

// Hive.go: returns all the currently managed peers that have been verified at least once.
pub(crate) fn get_verified_peers(active_peers: &ActivePeersList) -> Vec<ActivePeer> {
    let mut peers = Vec::with_capacity(active_peers.read().len());

    peers.extend(active_peers.read().iter().filter_map(|p| {
        if p.metrics().verified_count() > 0 {
            Some(p.clone())
        } else {
            None
        }
    }));

    peers
}
