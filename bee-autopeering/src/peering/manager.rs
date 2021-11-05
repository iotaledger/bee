// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    filter::NeighborFilter,
    messages::{DropPeeringRequest, PeeringRequest, PeeringResponse},
    neighbor::{self, Neighborhood, SIZE_INBOUND, SIZE_OUTBOUND},
};

use crate::{
    command::CommandTx,
    config::AutopeeringConfig,
    event::{Event, EventTx},
    local::{
        salt::{self, Salt, SALT_LIFETIME_SECS},
        services::AUTOPEERING_SERVICE_NAME,
        Local,
    },
    packet::{msg_hash, IncomingPacket, MessageType, OutgoingPacket},
    peer::{self, peer_id::PeerId, peerlist::ActivePeersList, Peer},
    peering::neighbor::{salt_distance, Neighbor},
    request::{self, RequestManager, RequestValue, ResponseTx, RESPONSE_TIMEOUT},
    server::{ServerSocket, ServerTx},
    task::{Repeat, Runnable, ShutdownRx},
    time::SECOND,
    NeighborValidator,
};

use std::{net::SocketAddr, time::Duration};

/// Salt update interval.
pub(crate) const SALT_UPDATE_SECS: Duration = Duration::from_secs(SALT_LIFETIME_SECS.as_secs() - SECOND);
const INCOMING: bool = true;
const OUTGOING: bool = false;

pub(crate) type InboundNeighborhood = Neighborhood<SIZE_INBOUND, true>;
pub(crate) type OutboundNeighborhood = Neighborhood<SIZE_OUTBOUND, false>;

/// Represents the answer of a `PeeringRequest`. Can be either `true` (peering accepted), or `false` (peering denied).
pub type Status = bool;

// TODO: revisit dead code
#[allow(dead_code)]
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

// TODO: revisit dead code
#[allow(dead_code)]
pub(crate) struct PeeringManagerConfig {
    pub(crate) version: u32,
    pub(crate) network_id: u32,
    pub(crate) source_addr: SocketAddr,
}

impl PeeringManagerConfig {
    pub fn new(config: &AutopeeringConfig, version: u32, network_id: u32) -> Self {
        Self {
            version,
            network_id,
            source_addr: config.bind_addr,
        }
    }
}

pub(crate) struct PeeringManager<V: NeighborValidator> {
    // The peering config.
    // TODO: revisit dead code
    #[allow(dead_code)]
    config: PeeringManagerConfig,
    // The local peer.
    local: Local,
    // Channel halfs for sending/receiving peering related packets.
    socket: ServerSocket,
    // Handles requests.
    request_mngr: RequestManager,
    // Publishes peering related events.
    event_tx: EventTx,
    // The list of managed peers.
    active_peers: ActivePeersList,
    // Inbound neighborhood.
    inbound_nbh: InboundNeighborhood,
    // Outbound neighborhood.
    outbound_nbh: OutboundNeighborhood,
    // The peer rejection filter.
    nb_filter: NeighborFilter<V>,
}

impl<V: NeighborValidator> PeeringManager<V> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        config: PeeringManagerConfig,
        local: Local,
        socket: ServerSocket,
        request_mngr: RequestManager,
        active_peers: ActivePeersList,
        event_tx: EventTx,
        // TODO: revisit dead code
        _command_tx: CommandTx,
        inbound_nbh: InboundNeighborhood,
        outbound_nbh: OutboundNeighborhood,
        nb_filter: NeighborFilter<V>,
    ) -> Self {
        Self {
            config,
            local,
            socket,
            request_mngr,
            event_tx,
            active_peers,
            inbound_nbh,
            outbound_nbh,
            nb_filter,
        }
    }
}

#[async_trait::async_trait]
impl<V: NeighborValidator> Runnable for PeeringManager<V> {
    const NAME: &'static str = "PeeringManager";
    const SHUTDOWN_PRIORITY: u8 = 1;

    type ShutdownSignal = ShutdownRx;

    async fn run(self, mut shutdown_rx: Self::ShutdownSignal) {
        let PeeringManager {
            // TODO: revisit dead code
            config: _,
            local,
            socket,
            request_mngr,
            event_tx,
            active_peers,
            inbound_nbh,
            outbound_nbh,
            nb_filter,
        } = self;

        let ServerSocket {
            mut server_rx,
            server_tx,
        } = socket;

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
                            active_peers: &active_peers,
                            request_mngr: &request_mngr,
                            peer_addr,
                            event_tx: &event_tx,
                            inbound_nbh: &inbound_nbh,
                            outbound_nbh: &outbound_nbh,
                        };

                        match msg_type {
                            MessageType::PeeringRequest => {
                                let peer_req = if let Ok(peer_req) = PeeringRequest::from_protobuf(&msg_bytes) {
                                    peer_req
                                } else {
                                    log::warn!("Error decoding peering request from {}.", &peer_id);
                                    continue 'recv;
                                };

                                if let Err(e) = validate_peering_request(&peer_req, &ctx) {
                                    log::warn!("Received invalid peering request from {}. Reason: {:?}", &peer_id, e);
                                    continue 'recv;
                                } else {
                                    log::trace!("Received valid peering request from {}.", &peer_id);

                                    handle_peering_request(peer_req, ctx, &nb_filter);
                                }
                            }
                            MessageType::PeeringResponse => {
                                let peer_res = if let Ok(peer_res) = PeeringResponse::from_protobuf(&msg_bytes) {
                                    peer_res
                                } else {
                                    log::warn!("Error decoding peering response from {}.", &peer_id);
                                    continue 'recv;
                                };

                                match validate_peering_response(&peer_res, &ctx) {
                                    Ok(peer_reqval) => {
                                        log::trace!("Received valid peering response from {}.", &peer_id);

                                        handle_peering_response(peer_res, peer_reqval, ctx, &nb_filter);
                                    }
                                    Err(e) => {
                                        log::warn!("Received invalid peering response from {}. Reason: {:?}", &peer_id, e);
                                        continue 'recv;
                                    }
                                }
                            }
                            MessageType::DropRequest => {
                                let drop_req = if let Ok(drop_req) = DropPeeringRequest::from_protobuf(&msg_bytes) {
                                    drop_req
                                } else {
                                    log::warn!("Error decoding drop request from {}.", &peer_id);
                                    continue 'recv;
                                };

                                if let Err(e) = validate_drop_request(&drop_req, &ctx) {
                                    log::warn!("Received invalid drop request from {}. Reason: {:?}", &peer_id, e);
                                    continue 'recv;
                                } else {
                                    log::trace!("Received valid drop request from {}.", &peer_id);

                                    handle_drop_request(drop_req, ctx, &nb_filter);
                                }
                            }
                            _ => log::warn!("Received unsupported peering message type"),
                        }
                    }
                }
            }
        }
    }
}

pub(crate) struct RecvContext<'a> {
    peer_id: &'a PeerId,
    msg_bytes: &'a [u8],
    server_tx: &'a ServerTx,
    local: &'a Local,
    active_peers: &'a ActivePeersList,
    request_mngr: &'a RequestManager,
    peer_addr: SocketAddr,
    event_tx: &'a EventTx,
    inbound_nbh: &'a InboundNeighborhood,
    outbound_nbh: &'a OutboundNeighborhood,
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// VALIDATION
///////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub(crate) enum ValidationError {
    // The request must not be expired.
    RequestExpired,
    // The response must arrive in time.
    NoCorrespondingRequestOrTimeout,
    // The hash of the corresponding request must be correct.
    IncorrectRequestHash,
    // The peer has not been verified yet.
    PeerNotVerified,
    // The peer's salt is expired.
    SaltExpired,
}

fn validate_peering_request(peer_req: &PeeringRequest, ctx: &RecvContext) -> Result<(), ValidationError> {
    use ValidationError::*;

    if request::is_expired(peer_req.timestamp()) {
        Err(RequestExpired)
    } else if !peer::is_verified(ctx.peer_id, ctx.active_peers) {
        Err(PeerNotVerified)
    } else if salt::is_expired(peer_req.salt().expiration_time()) {
        Err(SaltExpired)
    } else {
        Ok(())
    }
}

fn validate_peering_response(peer_res: &PeeringResponse, ctx: &RecvContext) -> Result<RequestValue, ValidationError> {
    use ValidationError::*;

    if let Some(reqv) = ctx.request_mngr.write().pull::<PeeringRequest>(ctx.peer_id) {
        if peer_res.request_hash() != reqv.request_hash {
            Err(IncorrectRequestHash)
        } else {
            Ok(reqv)
        }
    } else {
        Err(NoCorrespondingRequestOrTimeout)
    }
}

fn validate_drop_request(drop_req: &DropPeeringRequest, _: &RecvContext) -> Result<(), ValidationError> {
    use ValidationError::*;

    if request::is_expired(drop_req.timestamp()) {
        Err(RequestExpired)
    } else {
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// HANDLING
///////////////////////////////////////////////////////////////////////////////////////////////////////////

fn handle_peering_request<V: NeighborValidator>(
    peer_req: PeeringRequest,
    ctx: RecvContext,
    nb_filter: &NeighborFilter<V>,
) {
    log::trace!("Handling peering request.");

    // TODO: revisit peer salt
    let _peer_salt = peer_req.salt();

    let mut status = false;

    if peer::is_verified(ctx.peer_id, ctx.active_peers) {
        let active_peer = ctx
            .active_peers
            .read()
            .find(ctx.peer_id)
            .cloned()
            .expect("inconsistent peer list");

        if nb_filter.read().ok(&active_peer.peer()) {
            // if nb_validator.map_or(true, |v| v.is_valid(&active_peer.peer())) {
            // Calculate the distance between the local peer and the potential neighbor.
            let distance = neighbor::salt_distance(
                ctx.local.read().peer_id(),
                active_peer.peer_id(),
                ctx.local.read().private_salt().expect("missing private salt"),
            );

            // Create a new neighbor.
            let neighbor = Neighbor::new(active_peer.into_peer(), distance);

            // Check if the neighbor would be closer than the currently furthest in the inbound neighborhood.
            let mut guard = ctx.inbound_nbh.write();
            if let Some(peer) = guard.select(neighbor) {
                drop(guard);
                if add_or_replace_neighbor::<INCOMING>(
                    peer.clone(),
                    ctx.local,
                    ctx.inbound_nbh,
                    ctx.outbound_nbh,
                    ctx.server_tx,
                    ctx.event_tx,
                ) {
                    // Change peering status to `true`.
                    status = true;

                    // Update the neighbor filter.
                    nb_filter.write().add(*peer.peer_id());

                    // Fire `IncomingPeering` event.
                    publish_peering_event::<INCOMING>(
                        peer,
                        status,
                        ctx.local,
                        ctx.event_tx,
                        ctx.inbound_nbh,
                        ctx.outbound_nbh,
                    );
                }
            } else {
                log::debug!("Denying peering request: Peer distance too large.");
            }
        } else {
            log::debug!("Denying peering request: Peer filtered.");
        }
    } else {
        log::debug!("Denying peering request: Peer not verified.");
    }

    // In any case send a response.
    send_peering_response_to_addr(ctx.peer_addr, ctx.peer_id, ctx.msg_bytes, ctx.server_tx, status);
}

fn handle_peering_response<V: NeighborValidator>(
    peer_res: PeeringResponse,
    peer_reqval: RequestValue,
    ctx: RecvContext,
    nb_filter: &NeighborFilter<V>,
) {
    log::trace!("Handling peering response.");

    let mut status = peer_res.status();

    if status {
        log::debug!("Peering accepted by {}.", ctx.peer_id);

        let peer = ctx
            .active_peers
            .read()
            .find(ctx.peer_id)
            .cloned()
            .expect("inconsistent peer list")
            .into_peer();

        // Hive.go: if the peer is already in inbound, do not add it and remove it from inbound
        // TODO: investigate why!
        if ctx.inbound_nbh.write().remove_neighbor(ctx.peer_id).is_some() {
            // Change status to `false`.
            status = false;

            // Fire `OutgoingPeering` event with status = `false`.
            publish_peering_event::<OUTGOING>(
                peer.clone(),
                status,
                ctx.local,
                ctx.event_tx,
                ctx.inbound_nbh,
                ctx.outbound_nbh,
            );

            // Drop that peer.
            send_drop_peering_request_to_peer(peer, ctx.server_tx, ctx.event_tx, ctx.inbound_nbh, ctx.outbound_nbh);
        } else if ctx.outbound_nbh.write().insert_neighbor(peer.clone(), ctx.local) {
            // Update the neighbor filter.
            nb_filter.write().add(*peer.peer_id());

            // Fire `OutgoingPeering` event with status = `true`.
            publish_peering_event::<OUTGOING>(peer, status, ctx.local, ctx.event_tx, ctx.inbound_nbh, ctx.outbound_nbh);
        } else {
            log::debug!("Failed to add neighbor to outbound neighborhood after successful peering request");
        }
    } else {
        log::debug!("Peering by {} denied.", ctx.peer_id);
    }

    // Send the response notification.
    if let Some(tx) = peer_reqval.response_tx {
        tx.send(ctx.msg_bytes.to_vec()).expect("error sending response signal");
    }
}

fn handle_drop_request<V: NeighborValidator>(
    _drop_req: DropPeeringRequest,
    ctx: RecvContext,
    nb_filter: &NeighborFilter<V>,
) {
    log::trace!("Handling drop request.");

    let mut removed_nb = ctx.inbound_nbh.write().remove_neighbor(ctx.peer_id);

    if let Some(nb) = ctx.outbound_nbh.write().remove_neighbor(ctx.peer_id) {
        removed_nb.replace(nb);

        nb_filter.write().add(*ctx.peer_id);

        // TODO: trigger immediate outbound neighborhood update; currently we wait for the next interval
    }

    if removed_nb.is_some() {
        send_drop_peering_request_to_addr(
            ctx.peer_addr,
            *ctx.peer_id,
            ctx.server_tx,
            ctx.event_tx,
            ctx.inbound_nbh,
            ctx.outbound_nbh,
        );
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// SENDING
///////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Initiates a peering request.
///
/// This function is blocking, but at most for `RESPONSE_TIMEOUT` seconds.
pub(crate) async fn begin_peering(
    peer_id: &PeerId,
    active_peers: &ActivePeersList,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
    local: &Local,
) -> Option<Status> {
    let (response_tx, response_rx) = request::response_chan();

    send_peering_request_to_peer(peer_id, active_peers, request_mngr, server_tx, Some(response_tx), local);

    match tokio::time::timeout(RESPONSE_TIMEOUT, response_rx).await {
        Ok(Ok(bytes)) => {
            let peer_res = PeeringResponse::from_protobuf(&bytes).expect("error decoding peering response");
            Some(peer_res.status())
        }
        Ok(Err(e)) => {
            log::debug!("Response signal error: {}", e);
            None
        }
        Err(e) => {
            log::debug!("Peering response timeout: {}", e);
            None
        }
    }
}

/// Sends a peering request to a peer.
pub(crate) fn send_peering_request_to_peer(
    peer_id: &PeerId,
    active_peers: &ActivePeersList,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
    response_tx: Option<ResponseTx>,
    local: &Local,
) {
    // let peer = peerstore.fetch_peer(peer_id).expect("peer not in storage");
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

        send_peering_request_to_addr(peer_addr, peer_id, request_mngr, server_tx, response_tx, local);
    }
}

/// Sends a peering request to a peer's address.
pub(crate) fn send_peering_request_to_addr(
    peer_addr: SocketAddr,
    peer_id: &PeerId,
    request_mngr: &RequestManager,
    server_tx: &ServerTx,
    response_tx: Option<ResponseTx>,
    local: &Local,
) {
    log::trace!("Sending peering request to: {}", peer_id);

    let peer_req = request_mngr.write().new_peering_request(*peer_id, response_tx, local);

    let msg_bytes = peer_req.to_protobuf().expect("error encoding peering request").to_vec();

    server_tx
        .send(OutgoingPacket {
            msg_type: MessageType::PeeringRequest,
            msg_bytes,
            peer_addr,
        })
        .expect("error sending peering request to server");
}

/// Sends a peering response to a peer's address.
pub(crate) fn send_peering_response_to_addr(
    peer_addr: SocketAddr,
    peer_id: &PeerId,
    msg_bytes: &[u8],
    tx: &ServerTx,
    status: bool,
) {
    log::trace!("Sending peering response to: {}", peer_id);

    let request_hash = msg_hash(MessageType::PeeringRequest, msg_bytes).to_vec();

    let peer_res = PeeringResponse::new(request_hash, status);

    let msg_bytes = peer_res
        .to_protobuf()
        .expect("error encoding peering response")
        .to_vec();

    tx.send(OutgoingPacket {
        msg_type: MessageType::VerificationResponse,
        msg_bytes,
        peer_addr,
    })
    .expect("error sending peering response to server");
}

/// Sends a drop-peering request to a peer.
pub(crate) fn send_drop_peering_request_to_peer(
    peer: Peer,
    server_tx: &ServerTx,
    event_tx: &EventTx,
    inbound_nbh: &InboundNeighborhood,
    outbound_nbh: &OutboundNeighborhood,
) {
    let peer_port = peer
        .port(AUTOPEERING_SERVICE_NAME)
        .expect("missing autopeering service");
    let peer_addr = SocketAddr::new(peer.ip_address(), peer_port);
    let peer_id = peer.into_id();

    send_drop_peering_request_to_addr(peer_addr, peer_id, server_tx, event_tx, inbound_nbh, outbound_nbh);
}

/// Sends a drop-peering request to a peer's address.
pub(crate) fn send_drop_peering_request_to_addr(
    peer_addr: SocketAddr,
    peer_id: PeerId,
    server_tx: &ServerTx,
    event_tx: &EventTx,
    inbound_nbh: &InboundNeighborhood,
    outbound_nbh: &OutboundNeighborhood,
) {
    log::trace!("Sending drop request to: {}", peer_id);

    let msg_bytes = DropPeeringRequest::new()
        .to_protobuf()
        .expect("error encoding drop request")
        .to_vec();

    server_tx
        .send(OutgoingPacket {
            msg_type: MessageType::DropRequest,
            msg_bytes,
            peer_addr,
        })
        .expect("error sending drop-peering request to server");

    publish_drop_peering_event(peer_id, event_tx, inbound_nbh, outbound_nbh);
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// EVENTS
///////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Publishes the corresponding peering event [`IncomingPeering`], or [`OutgoingPeering`].
pub(crate) fn publish_peering_event<const IS_INBOUND: bool>(
    peer: Peer,
    status: Status,
    local: &Local,
    event_tx: &EventTx,
    inbound_nbh: &InboundNeighborhood,
    outbound_nbh: &OutboundNeighborhood,
) {
    log::debug!(
        "Peering requested => direction: {}, status: {}, to: {}, #out {}, #in {}",
        if IS_INBOUND { "in" } else { "out" },
        status,
        peer.peer_id(),
        outbound_nbh.read().len(),
        inbound_nbh.read().len(),
    );

    let distance = salt_distance(local.read().peer_id(), peer.peer_id(), &{
        if IS_INBOUND {
            local.read().private_salt().expect("missing private salt").clone()
        } else {
            local.read().public_salt().expect("missing public salt").clone()
        }
    });

    event_tx
        .send(if IS_INBOUND {
            Event::IncomingPeering { peer, distance }
        } else {
            Event::OutgoingPeering { peer, distance }
        })
        .expect("error publishing incoming/outgoing peering event");
}

fn publish_drop_peering_event(
    peer_id: PeerId,
    event_tx: &EventTx,
    inbound_nbh: &InboundNeighborhood,
    outbound_nbh: &OutboundNeighborhood,
) {
    log::debug!(
        "Peering dropped with {}; #out_nbh: {} #in_nbh: {}",
        peer_id,
        inbound_nbh.read().len(),
        outbound_nbh.read().len(),
    );

    event_tx
        .send(Event::PeeringDropped { peer_id })
        .expect("error sending peering-dropped event");
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// HELPERS
///////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub(crate) struct SaltUpdateContext<V: NeighborValidator> {
    local: Local,
    nb_filter: NeighborFilter<V>,
    inbound_nbh: InboundNeighborhood,
    outbound_nbh: OutboundNeighborhood,
    server_tx: ServerTx,
    event_tx: EventTx,
}

impl<V: NeighborValidator> SaltUpdateContext<V> {
    pub(crate) fn new(
        local: Local,
        nb_filter: NeighborFilter<V>,
        inbound_nbh: InboundNeighborhood,
        outbound_nbh: OutboundNeighborhood,
        server_tx: ServerTx,
        event_tx: EventTx,
    ) -> Self {
        Self {
            local,
            nb_filter,
            inbound_nbh,
            outbound_nbh,
            server_tx,
            event_tx,
        }
    }
}

// Regularly update the salts of the local peer.
pub(crate) fn repeat_update_salts<V: NeighborValidator>(
    drop_neighbors_on_salt_update: bool,
) -> Repeat<SaltUpdateContext<V>> {
    Box::new(move |ctx| {
        update_salts(
            drop_neighbors_on_salt_update,
            &ctx.local,
            &ctx.nb_filter,
            &ctx.inbound_nbh,
            &ctx.outbound_nbh,
            &ctx.server_tx,
            &ctx.event_tx,
        )
    })
}

fn update_salts<V: NeighborValidator>(
    drop_neighbors_on_salt_update: bool,
    local: &Local,
    nb_filter: &NeighborFilter<V>,
    inbound_nbh: &InboundNeighborhood,
    outbound_nbh: &OutboundNeighborhood,
    server_tx: &ServerTx,
    event_tx: &EventTx,
) {
    // Create a new private salt.
    let private_salt = Salt::new(SALT_LIFETIME_SECS);
    let private_salt_lifetime = private_salt.expiration_time();
    local.write().set_private_salt(private_salt);

    // Create a new public salt.
    let public_salt = Salt::new(SALT_LIFETIME_SECS);
    let public_salt_lifetime = public_salt.expiration_time();
    local.write().set_public_salt(public_salt);

    if drop_neighbors_on_salt_update {
        // Drop all neighbors.
        for peer in inbound_nbh.read().iter().chain(outbound_nbh.read().iter()).cloned() {
            send_drop_peering_request_to_peer(peer, server_tx, event_tx, inbound_nbh, outbound_nbh);
        }

        // Erase the neighborhoods.
        inbound_nbh.write().clear();
        outbound_nbh.write().clear();

        // Reset the neighbor filter.
        nb_filter.write().reset();
    } else {
        // Update the distances with the new salts.
        inbound_nbh.write().update_distances(local);
        outbound_nbh.write().update_distances(local);
    }

    log::debug!(
        "Salts updated; private: {}, public: {}",
        private_salt_lifetime,
        public_salt_lifetime,
    );

    // Publish 'SaltUpdated' event.
    event_tx
        .send(Event::SaltUpdated {
            public_salt_lifetime,
            private_salt_lifetime,
        })
        .expect("error publishing salt-updated event");
}

/// Adds a neighbor to a neighborhood. Possibly even replaces the so far furthest neighbor.
pub(crate) fn add_or_replace_neighbor<const IS_INBOUND: bool>(
    peer: Peer,
    local: &Local,
    inbound_nbh: &InboundNeighborhood,
    outbound_nbh: &OutboundNeighborhood,
    server_tx: &ServerTx,
    event_tx: &EventTx,
) -> bool {
    // Hive.go: drop furthest neighbor if necessary
    if let Some(peer) = if IS_INBOUND {
        inbound_nbh.write().remove_furthest()
    } else {
        outbound_nbh.write().remove_furthest()
    } {
        send_drop_peering_request_to_peer(peer, server_tx, event_tx, inbound_nbh, outbound_nbh);
    }

    if IS_INBOUND {
        inbound_nbh.write().insert_neighbor(peer, local)
    } else {
        outbound_nbh.write().insert_neighbor(peer, local)
    }
}

// TODO: revisit dead code
/// Reinitializes the neighbor filter with the current neighborhoods.
///
/// Call this function whenever one of the neighborhoods changes.
#[allow(dead_code)]
pub(crate) fn refresh_neighbor_filter<V: NeighborValidator>(
    neighbor_filter: &NeighborFilter<V>,
    inbound_nbh: &InboundNeighborhood,
    outbound_nbh: &OutboundNeighborhood,
) {
    let mut write = neighbor_filter.write();

    write.reset();
    write.extend(
        inbound_nbh
            .read()
            .iter()
            .map(|p| p.peer_id())
            .copied()
            .chain(outbound_nbh.read().iter().map(|p| p.peer_id()).copied()),
    );
}
