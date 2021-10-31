// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    command::{self, Command, CommandRx, CommandTx},
    config::AutopeeringConfig,
    discovery::messages::{DiscoveryRequest, DiscoveryResponse, VerificationRequest, VerificationResponse},
    event::{Event, EventTx},
    local::{
        service_map::{ServiceMap, ServicePort, ServiceTransport, AUTOPEERING_SERVICE_NAME},
        Local,
    },
    multiaddr::{AddressKind, AutopeeringMultiaddr},
    packet::{msg_hash, IncomingPacket, MessageType, OutgoingPacket},
    peer::{
        self,
        peer_id::PeerId,
        peerlist::{ActivePeerEntry, ActivePeersList, MasterPeersList, ReplacementList},
        peerstore::PeerStore,
        Peer,
    },
    request::{self, RequestManager, RequestValue, ResponseTx, RESPONSE_TIMEOUT},
    server::{ServerRx, ServerSocket, ServerTx},
    task::{Runnable, ShutdownRx, TaskManager},
    time::{self, SECOND},
};

use rand::{seq::index, Rng as _};
use tokio::sync::oneshot;

use std::net::{IpAddr, SocketAddr};

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

pub(crate) type DiscoveryHandler<S: PeerStore> = Box<dyn Fn(&RecvContext<S>) + Send + Sync + 'static>;

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

pub(crate) struct DiscoveryManager<S: PeerStore> {
    // Config.
    config: DiscoveryManagerConfig,
    // The local id to sign outgoing packets.
    local: Local,
    // Channel halfs for sending/receiving discovery related packets.
    socket: ServerSocket,
    // Handles requests.
    request_mngr: RequestManager<S>,
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
        request_mngr: RequestManager<S>,
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

    pub async fn init<const N: usize>(self, task_mngr: &mut TaskManager<N>) -> CommandTx {
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
            run_as_entry_node,
            version,
            network_id,
            bind_addr,
        } = config;

        let ServerSocket {
            mut server_rx,
            server_tx,
        } = socket;

        add_master_peers(
            &mut entry_nodes,
            entry_nodes_prefer_ipv6,
            &request_mngr,
            &server_tx,
            &local,
            &peerstore,
            &master_peers,
            &active_peers,
            &replacements,
        )
        .await;

        restore_peers(&peerstore, &local, &active_peers, &replacements);

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

        task_mngr.run::<DiscoveryRecvHandler<S>>(discovery_recv_handler);
        task_mngr.run::<DiscoverySendHandler<S>>(discovery_send_handler);

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
    request_mngr: RequestManager<S>,
    event_tx: EventTx,
    active_peers: ActivePeersList,
    replacements: ReplacementList,
}

struct DiscoverySendHandler<S: PeerStore> {
    server_tx: ServerTx,
    local: Local,
    peerstore: S,
    request_mngr: RequestManager<S>,
    active_peers: ActivePeersList,
    command_rx: CommandRx,
}

#[async_trait::async_trait]
impl<S: PeerStore> Runnable for DiscoveryRecvHandler<S> {
    const NAME: &'static str = "DiscoveryRecvHandler";
    const SHUTDOWN_PRIORITY: u8 = 4;

    type ShutdownSignal = ShutdownRx;

    async fn run(self, mut shutdown_rx: Self::ShutdownSignal) {
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
            replacements,
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
                            peerstore: &peerstore,
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

                                match validate_verification_response(&verif_res, &request_mngr, &peer_id, peer_addr) {
                                    Ok(verif_reqval) => {
                                        log::debug!("Received valid verification response from {}.", &peer_id);

                                        handle_verification_response(verif_res, verif_reqval, ctx);
                                    }
                                    Err(e) => {
                                        log::debug!("Received invalid verification response from {}. Reason: {:?}", &peer_id, e);
                                        continue 'recv;
                                    }
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

                                match validate_discovery_response(&disc_res, &request_mngr, &peer_id) {
                                    Ok(disc_reqval) => {
                                        log::debug!("Received valid discovery response from {}.", &peer_id);

                                        handle_discovery_response(disc_res, disc_reqval, ctx);
                                    }
                                    Err(e) => {
                                        log::debug!("Received invalid discovery response from {}. Reason: {:?}", &peer_id, e);
                                        continue 'recv;
                                    }
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
    const SHUTDOWN_PRIORITY: u8 = 5;

    type ShutdownSignal = ShutdownRx;

    async fn run(self, mut shutdown_rx: Self::ShutdownSignal) {
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
                                send_verification_request_to_peer(&peer_id, &request_mngr, &peerstore, &server_tx, None);
                            }
                            Command::SendDiscoveryRequest { peer_id } => {
                                send_discovery_request_to_peer(&peer_id, &request_mngr, &peerstore, &server_tx, None);
                            }
                            Command::SendVerificationRequests => {
                                // TODO: For testing purposes only => remove it at some point
                                send_verification_request_to_all(&active_peers, &request_mngr, &peerstore, &server_tx);
                            }
                            Command::SendDiscoveryRequests => {
                                // TODO: For testing purposes only => remove it at some point
                                send_discovery_request_to_all_verified(&active_peers, &request_mngr, &peerstore, &server_tx);
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn add_master_peers<S: PeerStore>(
    entry_nodes: &mut Vec<AutopeeringMultiaddr>,
    entry_nodes_prefer_ipv6: bool,
    request_mngr: &RequestManager<S>,
    server_tx: &ServerTx,
    local: &Local,
    peerstore: &S,
    master_peers: &MasterPeersList,
    active_peers: &ActivePeersList,
    replacements: &ReplacementList,
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

        master_peers.write().insert(peer.peer_id().clone());

        if add_peer(peer, local, active_peers, replacements, peerstore) {
            num_added += 1;
        }
    }

    log::debug!("Added {} entry node/s.", num_added);
}

/// Attempts to add a peer to the peer list and/or peer store.
///
/// It won't add the peer, if:
/// * the peer is the local peer
/// * the peer is known to the list and the store already
pub(crate) fn add_peer<S: PeerStore>(
    peer: Peer,
    local: &Local,
    active_peers: &ActivePeersList,
    replacements: &ReplacementList,
    peerstore: &S,
) -> bool {
    let peer_id = peer.peer_id().clone();

    let added_to_list = add_peer_to_list(peer.clone(), local, active_peers, replacements);
    let added_to_store = add_peer_to_store(peer, local, peerstore);

    // Return true if it was added "somewhere".
    if added_to_list || added_to_store {
        log::debug!("Peer added: {}", peer_id);
        true
    } else {
        false
    }
}

// From hive.go:
// Adds a newly discovered peer that has never been verified or pinged yet.
// It returns true, if the given peer was new and added, false otherwise.
fn add_peer_to_list(peer: Peer, local: &Local, active_peers: &ActivePeersList, replacements: &ReplacementList) -> bool {
    if peer.peer_id() == local.read().peer_id() {
        // Do not add the local peer.
        false
    } else if active_peers.read().contains(peer.peer_id()) {
        // Do not add it if it already exists.
        false
    } else {
        // Prefer adding it to the active list, but if it is already full add the peer to the replacement list.
        if !active_peers.read().is_full() {
            active_peers.write().insert(peer.into())
        } else {
            replacements.write().insert(peer)
        }
    }
}

// Adds a new peer to the peer store.
fn add_peer_to_store<S: PeerStore>(peer: Peer, local: &Local, peerstore: &S) -> bool {
    if peer.peer_id() != local.read().peer_id() {
        peerstore.store_peer(peer)
    } else {
        false
    }
}

fn restore_peers<S: PeerStore>(
    peerstore: &S,
    local: &Local,
    active_peers: &ActivePeersList,
    replacements: &ReplacementList,
) {
    let mut num_added = 0;

    // TODO: instead of keep adding thousands of peers from the peer store, we should just stop
    // once the replacement list is also filled.
    for peer in peerstore.fetch_peers() {
        if add_peer_to_list(peer, &local, active_peers, replacements) {
            num_added += 1;
        }
    }

    log::debug!("Added {} stored peer/s.", num_added);
}

// Note: this function is dead-lock danger zone!
/// Deletes a peer from the active peerlist if it's not a master peer, and replaces it by a peer
/// from the replacement list.
#[rustfmt::skip]
pub(crate) fn remove_peer_from_active_list<S: PeerStore>(
    peer_id: &PeerId,
    master_peers: &MasterPeersList,
    active_peers: &ActivePeersList,
    replacements: &ReplacementList,
    peerstore: &S,
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
                    .send(Event::PeerDeleted {
                        peer_id: peer_id.clone(),
                    })
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

/// Determines whether the peer id is known.
///
/// Note, that this function doesn't query the peer store. This might change in the future.
fn is_known_peer_id_in_memory(
    peer_id: &PeerId,
    local: &Local,
    active_peers: &ActivePeersList,
    replacements: &ReplacementList,
) -> bool {
    // The master list doesn't need to be queried, because those always a subset of the active peers.
    peer_id == local.read().peer_id() || active_peers.read().contains(&peer_id) || replacements.read().contains(peer_id)
}

// TODO: Refactor this function as it does and checks too many things, and is hard to understand (likely cause for
// bugs).
/// Hive.go: adds a new peer that has just been successfully pinged.
/// It returns true, if the given peer was new and added, false otherwise.
///
/// Fires the `PeerDiscovered` event.
fn add_verified_peer(
    peer: Peer,
    local: &Local,
    active_peers: &ActivePeersList,
    replacements: &ReplacementList,
    event_tx: &EventTx,
) -> bool {
    // Hive.go: never add the local peer
    if local.read().peer_id() == peer.peer_id() {
        false
    } else if let Some(verified_count) = update_peer_verified_count(peer.peer_id(), active_peers) {
        if verified_count == 1 {
            // Hive.go: trigger the event only for the first time the peer is updated
            event_tx.send(Event::PeerDiscovered {
                peer_id: peer.peer_id().clone(),
            });
        }
        false
    } else {
        // If the active/managed peer list is already full, then try to add the peer id to the replacement list
        if active_peers.read().is_full() {
            replacements.write().insert(peer)
        } else {
            let peer_id = peer.peer_id().clone();

            // Create a new `ActivePeerEntry` with `verified_count` set to 1.
            let mut active_peer = ActivePeerEntry::new(peer);
            active_peer.metrics_mut().increment_verified_count();

            // Hive.go: new nodes are added to the front
            if active_peers.write().insert(active_peer) {
                // Hive.go: trigger the event only when the peer is added to active
                event_tx.send(Event::PeerDiscovered { peer_id });
                true
            } else {
                false
            }
        }
    }
}

/// Hive.go: moves the peer with the given ID to the front of the list of managed peers.
/// It returns `None` if there was no peer with that id, otherwise the `verified_count` of the updated peer is returned.
fn update_peer_verified_count(peer_id: &PeerId, active_peers: &ActivePeersList) -> Option<usize> {
    if let Some(p) = active_peers.write().set_newest_and_get_mut(&peer_id) {
        let verified_count = p.metrics_mut().increment_verified_count();
        Some(verified_count)
    } else {
        None
    }
}

/// Hive.go: adds a newly discovered peer that has never been verified or pinged yet.
/// It returns true, if the given peer was new and added, false otherwise.
fn add_discovered_peer(
    peer: Peer,
    local: &Local,
    active_peers: &ActivePeersList,
    replacements: &ReplacementList,
) -> bool {
    if local.read().peer_id() == peer.peer_id() {
        // Hive.go: never add the local peer
        false
    } else if active_peers.read().contains(peer.peer_id()) {
        // Do not add it if it already exists.
        false
    } else {
        // Prefer adding it to the active list, but if it is already full, add the peer to the replacement list.
        let active_peer = ActivePeerEntry::new(peer);
        if !active_peers.read().is_full() {
            active_peers.write().insert(active_peer)
        } else {
            replacements.write().insert(active_peer.into_peer())
        }
    }
}

pub(crate) struct RecvContext<'a, S: PeerStore> {
    peer_id: &'a PeerId,
    msg_bytes: &'a [u8],
    server_tx: &'a ServerTx,
    local: &'a Local,
    peerstore: &'a S,
    request_mngr: &'a RequestManager<S>,
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

fn validate_verification_response<S: PeerStore>(
    verif_res: &VerificationResponse,
    request_mngr: &RequestManager<S>,
    peer_id: &PeerId,
    source_socket_addr: SocketAddr,
) -> Result<RequestValue<S>, ValidationError> {
    use ValidationError::*;

    if let Some(reqv) = request_mngr.write().pull::<VerificationRequest>(peer_id) {
        if verif_res.request_hash() == &reqv.request_hash {
            let res_services = verif_res.services();
            if let Some(res_peering) = res_services.get(AUTOPEERING_SERVICE_NAME) {
                if res_peering.port() == source_socket_addr.port() {
                    Ok(reqv)
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

fn validate_discovery_response<S: PeerStore>(
    disc_res: &DiscoveryResponse,
    request_mngr: &RequestManager<S>,
    peer_id: &PeerId,
) -> Result<RequestValue<S>, ValidationError> {
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

fn handle_verification_request<S: PeerStore>(verif_req: VerificationRequest, ctx: RecvContext<S>) {
    log::debug!("Handling verification request.");

    ctx.peerstore.update_last_verification_request(ctx.peer_id.clone());

    send_verification_response_to_addr(ctx.peer_addr, &verif_req, ctx.msg_bytes, ctx.server_tx, ctx.local);

    // TODO: the 2nd if-branch is a bit confusing: How come, that a peer can be unknown, but is verified?
    // ```go
    // if the peer is unknown or expired, send a Ping to verify
    // if !p.IsVerified(from.ID(), dstAddr.IP) {
    //     p.sendPing(dstAddr, from.ID())
    // } else if !p.mgr.isKnown(from.ID()) {
    //     // add a discovered peer to the manager if it is new but verified
    // 	   p.mgr.addDiscoveredPeer(newPeer(from, s.LocalAddr().Network(), dstAddr))
    // }
    // ```
    if !peer::is_verified(ctx.peer_id, ctx.peerstore) {
        send_verification_request_to_addr(ctx.peer_addr, ctx.peer_id, ctx.request_mngr, ctx.server_tx, None);
    } else if !is_known_peer_id_in_memory(ctx.peer_id, ctx.local, ctx.active_peers, ctx.replacements) {
        // Hive.go: add a discovered peer to the manager if it is new/unknown but verified
        // add_discovered_peer(peer, ctx.local, ctx.active_peers, ctx.replacements);
        unreachable!("unknown peer which is verified");
    }
}

fn handle_verification_response<S: PeerStore>(
    verif_res: VerificationResponse,
    verif_reqval: RequestValue<S>,
    ctx: RecvContext<S>,
) {
    log::debug!("Handling verification response.");

    // Execute the stored handler. Do nothing by default.
    (verif_reqval.handler.unwrap_or(Box::new(|ctx| {})))(&ctx);

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

fn handle_discovery_request<S: PeerStore>(disc_req: DiscoveryRequest, ctx: RecvContext<S>) {
    log::debug!("Handling discovery request.");

    let request_hash = msg_hash(MessageType::DiscoveryRequest, ctx.msg_bytes).to_vec();

    // TODO: prevent the `PeerId` clone in this method, and instead pass a closure that pulls the corresponding peers
    // from the storage.
    let chosen_peers = choose_n_random_active_peers(ctx.active_peers, MAX_PEERS_IN_RESPONSE, MIN_VERIFIED_IN_RESPONSE);

    let mut peers = Vec::with_capacity(MAX_PEERS_IN_RESPONSE);
    peers.extend(
        chosen_peers
            .into_iter()
            .filter_map(|peer_id| ctx.peerstore.fetch_peer(&peer_id)),
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
            peer_addr: ctx.peer_addr,
        })
        .expect("error sending verification response to server");
}

fn handle_discovery_response<S: PeerStore>(
    disc_res: DiscoveryResponse,
    disc_reqval: RequestValue<S>,
    ctx: RecvContext<S>,
) {
    // Remove the corresponding request from the request manager.
    log::debug!("Handling discovery response.");

    // Execute the stored handler if there's any.
    (disc_reqval.handler.unwrap_or(Box::new(|_| {})))(&ctx);

    // Add discovered peers to the peer list and peer store.
    for peer in disc_res.into_peers() {
        let peer_id = peer.peer_id().clone();

        if add_peer(peer, ctx.local, ctx.active_peers, ctx.replacements, ctx.peerstore) {
            log::debug!("Peer discovered: {}", peer_id);

            ctx.event_tx.send(Event::PeerDiscovered { peer_id });
        }
    }

    // Send the response notification.
    if let Some(tx) = disc_reqval.response_tx {
        tx.send(ctx.msg_bytes.to_vec()).expect("error sending response signal");
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// SENDING
///////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Initiates a verification request to a peer by fetching its endpoint data from the peer store and waiting
/// for the peer's response, which must arrive in certain amount of time.
///
/// Returns `Some(ServiceMap)` if the request was successful, otherwise `None`.
pub(crate) async fn begin_verification_request<S: PeerStore>(
    peer_id: &PeerId,
    request_mngr: &RequestManager<S>,
    peerstore: &S,
    server_tx: &ServerTx,
) -> Option<ServiceMap> {
    let (response_tx, response_rx) = request::response_chan();

    send_verification_request_to_peer(peer_id, request_mngr, peerstore, server_tx, Some(response_tx));

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

/// Sends a verification request to a peer by fetching its endpoint data from the peer store.
///
/// The function is non-blocking.
pub(crate) fn send_verification_request_to_peer<S: PeerStore>(
    peer_id: &PeerId,
    request_mngr: &RequestManager<S>,
    peerstore: &S,
    server_tx: &ServerTx,
    response_tx: Option<ResponseTx>,
) {
    let peer = peerstore.fetch_peer(peer_id).expect("peer not in storage");

    let peer_port = peer
        .services()
        .get(AUTOPEERING_SERVICE_NAME)
        .expect("peer doesn't support autopeering")
        .port();

    let peer_addr = SocketAddr::new(peer.ip_address(), peer_port);

    send_verification_request_to_addr(peer_addr, peer_id, request_mngr, server_tx, response_tx);
}

/// Sends a verification request to a peer's address.
pub(crate) fn send_verification_request_to_addr<S: PeerStore>(
    peer_addr: SocketAddr,
    peer_id: &PeerId,
    request_mngr: &RequestManager<S>,
    server_tx: &ServerTx,
    response_tx: Option<ResponseTx>,
) {
    log::debug!("Sending verification request to: {}", peer_id);

    // Define what happens when we receive the corresponding response.
    let handler = Box::new(|ctx: &RecvContext<S>| {
        log::trace!("Executing verification handler.");

        // Fetch the peer from the peer store.
        let peer = ctx.peerstore.fetch_peer(ctx.peer_id).expect("peer not stored");

        // This will increase the verified count of that peer, and move it to the front of the list.
        add_verified_peer(peer, ctx.local, ctx.active_peers, ctx.replacements, ctx.event_tx);

        // Update verification timestamp.
        ctx.peerstore.update_last_verification_response(ctx.peer_id.clone());
    });

    let verif_req =
        request_mngr
            .write()
            .new_verification_request(peer_id.clone(), peer_addr.ip(), Some(handler), response_tx);

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
    verif_req: &VerificationRequest,
    msg_bytes: &[u8],
    tx: &ServerTx,
    local: &Local,
) {
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
    tx.send(OutgoingPacket {
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
pub(crate) async fn begin_discovery_request<S: PeerStore>(
    peer_id: &PeerId,
    request_mngr: &RequestManager<S>,
    peerstore: &S,
    server_tx: &ServerTx,
) -> Option<Vec<Peer>> {
    let (response_tx, response_rx) = request::response_chan();

    send_discovery_request_to_peer(peer_id, request_mngr, peerstore, server_tx, Some(response_tx));

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
pub(crate) fn send_discovery_request_to_peer<S: PeerStore>(
    peer_id: &PeerId,
    request_mngr: &RequestManager<S>,
    peerstore: &S,
    server_tx: &ServerTx,
    response_tx: Option<ResponseTx>,
) {
    let peer = peerstore.fetch_peer(peer_id).expect("peer not in storage");

    let peer_port = peer
        .services()
        .get(AUTOPEERING_SERVICE_NAME)
        .expect("peer doesn't support autopeering")
        .port();

    let peer_addr = SocketAddr::new(peer.ip_address(), peer_port);

    send_discovery_request_to_addr(peer_addr, peer_id, request_mngr, peerstore, server_tx, response_tx);
}

/// Sends a discovery request to a peer's address.
pub(crate) fn send_discovery_request_to_addr<S: PeerStore>(
    peer_addr: SocketAddr,
    peer_id: &PeerId,
    request_mngr: &RequestManager<S>,
    peerstore: &S,
    server_tx: &ServerTx,
    response_tx: Option<ResponseTx>,
) {
    log::debug!("Sending discovery request to: {:?}", peer_id);

    let handler = None;

    let disc_req = request_mngr
        .write()
        .new_discovery_request(peer_id.clone(), handler, response_tx);

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

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// HELPERS
///////////////////////////////////////////////////////////////////////////////////////////////////////////

fn choose_n_random_active_peers(
    active_peers: &ActivePeersList,
    mut n: usize,
    min_verified_count: usize,
) -> Vec<PeerId> {
    let len = active_peers.read().len();

    if active_peers.read().len() <= n {
        // No randomization required => return all we got - if possible.
        let mut all_peers = Vec::with_capacity(len);
        all_peers.extend(active_peers.read().iter().filter_map(|entry| {
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
                .map(|index| active_peers.read().get(index).unwrap().clone())
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

// TEMP: for testing purposes
pub(crate) fn send_verification_request_to_all<S: PeerStore>(
    active_peers: &ActivePeersList,
    request_mngr: &RequestManager<S>,
    peerstore: &S,
    server_tx: &ServerTx,
) {
    log::debug!("Sending verification request to all peers.");

    for active_peer in active_peers.read().iter() {
        send_verification_request_to_peer(active_peer.peer_id(), request_mngr, peerstore, server_tx, None);
    }
}

// TEMP: for testing purposes
pub(crate) fn send_discovery_request_to_all_verified<S: PeerStore>(
    active_peers: &ActivePeersList,
    request_mngr: &RequestManager<S>,
    peerstore: &S,
    server_tx: &ServerTx,
) {
    log::debug!("Sending discovery request to all verified peers.");

    for active_peer in active_peers.read().iter() {
        if peer::is_verified(active_peer.peer_id(), peerstore) {
            send_discovery_request_to_peer(active_peer.peer_id(), request_mngr, peerstore, server_tx, None);
        }
    }
}
