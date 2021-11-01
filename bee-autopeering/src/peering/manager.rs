// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    filter::ExclusionFilter,
    messages::{DropPeeringRequest, PeeringRequest, PeeringResponse},
    neighbor::{Neighborhood, SIZE_INBOUND, SIZE_OUTBOUND},
    protocol::Direction,
};

use crate::{
    command::CommandTx,
    config::AutopeeringConfig,
    delay::ManualDelayFactory,
    discovery,
    event::{Event, EventTx},
    hash,
    local::{
        salt::{self, Salt, SALT_LIFETIME_SECS},
        service_map::AUTOPEERING_SERVICE_NAME,
        Local,
    },
    packet::{msg_hash, IncomingPacket, MessageType, OutgoingPacket},
    peer::{self, peer_id::PeerId, peerlist::ActivePeersList, peerstore::PeerStore, Peer},
    peering::neighbor::{salted_distance, NeighborDistance},
    request::{self, RequestManager, RequestValue, ResponseTx, RESPONSE_TIMEOUT},
    server::{ServerSocket, ServerTx},
    task::{Repeat, Runnable, ShutdownRx},
    time::{MINUTE, SECOND},
    NeighborValidator,
};

use std::{fmt::Display, net::SocketAddr, time::Duration, vec};

/// Salt update interval.
pub(crate) const SALT_UPDATE_SECS: Duration = Duration::from_secs(SALT_LIFETIME_SECS.as_secs() - SECOND);

pub(crate) type InboundNeighborhood = Neighborhood<SIZE_INBOUND, true>;
pub(crate) type OutboundNeighborhood = Neighborhood<SIZE_OUTBOUND, false>;

pub(crate) type PeeringHandler<S: PeerStore> = Box<dyn Fn(&RecvContext<S>) + Send + Sync + 'static>;

pub type Status = bool;

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

pub(crate) struct PeeringManager<S: PeerStore, V: NeighborValidator> {
    // The peering config.
    config: PeeringManagerConfig,
    // The local peer.
    local: Local,
    // Channel halfs for sending/receiving peering related packets.
    socket: ServerSocket,
    // Handles requests.
    request_mngr: RequestManager<S>,
    // Publishes peering related events.
    event_tx: EventTx,
    // A user defined storage layer for discovered peers.
    peerstore: S,
    // The list of managed peers.
    active_peers: ActivePeersList,
    // Inbound neighborhood.
    inbound_nbh: InboundNeighborhood,
    // Outbound neighborhood.
    outbound_nbh: OutboundNeighborhood,
    // The peer rejection filter.
    excl_filter: ExclusionFilter,
    // A user defined neighbor validator.
    neighbor_validator: V,
}

impl<S: PeerStore, V: NeighborValidator> PeeringManager<S, V> {
    pub(crate) fn new(
        config: PeeringManagerConfig,
        local: Local,
        socket: ServerSocket,
        request_mngr: RequestManager<S>,
        peerstore: S,
        active_peers: ActivePeersList,
        event_tx: EventTx,
        command_tx: CommandTx,
        neighbor_validator: V,
    ) -> Self {
        Self {
            config,
            local,
            socket,
            request_mngr,
            event_tx,
            peerstore,
            active_peers,
            inbound_nbh: Neighborhood::new(),
            outbound_nbh: Neighborhood::new(),
            excl_filter: ExclusionFilter::new(),
            neighbor_validator,
        }
    }
}

#[async_trait::async_trait]
impl<S: PeerStore + 'static, V: NeighborValidator> Runnable for PeeringManager<S, V> {
    const NAME: &'static str = "PeeringManager";
    const SHUTDOWN_PRIORITY: u8 = 1;

    type ShutdownSignal = ShutdownRx;

    async fn run(self, mut shutdown_rx: Self::ShutdownSignal) {
        let PeeringManager {
            config,
            local,
            socket,
            request_mngr,
            event_tx,
            peerstore,
            active_peers,
            inbound_nbh,
            outbound_nbh,
            excl_filter,
            neighbor_validator,
        } = self;

        let PeeringManagerConfig {
            version,
            network_id,
            source_addr,
        } = config;

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
                            peerstore: &peerstore,
                            active_peers: &active_peers,
                            request_mngr: &request_mngr,
                            peer_addr,
                            event_tx: &event_tx,
                            inbound_nbh: &inbound_nbh,
                            outbound_nbh: &outbound_nbh,
                            excl_filter: &excl_filter,
                        };

                        match msg_type {
                            MessageType::PeeringRequest => {
                                let peer_req = if let Ok(peer_req) = PeeringRequest::from_protobuf(&msg_bytes) {
                                    peer_req
                                } else {
                                    log::debug!("Error decoding peering request from {}.", &peer_id);
                                    continue 'recv;
                                };

                                if let Err(e) = validate_peering_request(&peer_req, &ctx) {
                                    log::debug!("Received invalid peering request from {}. Reason: {:?}", &peer_id, e);
                                    continue 'recv;
                                } else {
                                    log::debug!("Received valid peering request from {}.", &peer_id);

                                    handle_peering_request(peer_req, ctx);
                                }
                            }
                            MessageType::PeeringResponse => {
                                let peer_res = if let Ok(peer_res) = PeeringResponse::from_protobuf(&msg_bytes) {
                                    peer_res
                                } else {
                                    log::debug!("Error decoding peering response from {}.", &peer_id);
                                    continue 'recv;
                                };

                                match validate_peering_response(&peer_res, &ctx) {
                                    Ok(peer_reqval) => {
                                        log::debug!("Received valid peering response from {}.", &peer_id);

                                        handle_peering_response(peer_res, peer_reqval, ctx);
                                    }
                                    Err(e) => {
                                        log::debug!("Received invalid peering response from {}. Reason: {:?}", &peer_id, e);
                                        continue 'recv;
                                    }
                                }
                            }
                            MessageType::DropRequest => {
                                let drop_req = if let Ok(drop_req) = DropPeeringRequest::from_protobuf(&msg_bytes) {
                                    drop_req
                                } else {
                                    log::debug!("Error decoding drop request from {}.", &peer_id);
                                    continue 'recv;
                                };

                                if let Err(e) = validate_drop_request(&drop_req, &ctx) {
                                    log::debug!("Received invalid drop request from {}. Reason: {:?}", &peer_id, e);
                                    continue 'recv;
                                } else {
                                    log::debug!("Received valid drop request from {}.", &peer_id);

                                    handle_drop_request(drop_req, ctx);
                                }
                            }
                            _ => log::debug!("Received unsupported peering message type"),
                        }
                    }
                }
            }
        }
    }
}

pub(crate) struct RecvContext<'a, S: PeerStore> {
    peer_id: &'a PeerId,
    msg_bytes: &'a [u8],
    server_tx: &'a ServerTx,
    local: &'a Local,
    peerstore: &'a S,
    active_peers: &'a ActivePeersList,
    request_mngr: &'a RequestManager<S>,
    peer_addr: SocketAddr,
    event_tx: &'a EventTx,
    inbound_nbh: &'a InboundNeighborhood,
    outbound_nbh: &'a OutboundNeighborhood,
    excl_filter: &'a ExclusionFilter,
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

fn validate_peering_request<S: PeerStore>(
    peer_req: &PeeringRequest,
    ctx: &RecvContext<S>,
) -> Result<(), ValidationError> {
    use ValidationError::*;

    if request::is_expired(peer_req.timestamp()) {
        Err(RequestExpired)
    } else if !peer::is_verified(ctx.peer_id, ctx.peerstore) {
        Err(PeerNotVerified)
    } else if salt::is_expired(peer_req.salt().expiration_time()) {
        Err(SaltExpired)
    } else {
        Ok(())
    }
}

fn validate_peering_response<S: PeerStore>(
    peer_res: &PeeringResponse,
    ctx: &RecvContext<S>,
) -> Result<RequestValue<S>, ValidationError> {
    use ValidationError::*;

    if let Some(reqv) = ctx.request_mngr.write().pull::<PeeringRequest>(ctx.peer_id) {
        if peer_res.request_hash() != &reqv.request_hash {
            Err(IncorrectRequestHash)
        } else {
            Ok(reqv)
        }
    } else {
        Err(NoCorrespondingRequestOrTimeout)
    }
}

fn validate_drop_request<S: PeerStore>(
    drop_req: &DropPeeringRequest,
    _: &RecvContext<S>,
) -> Result<(), ValidationError> {
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

fn handle_peering_request<S: PeerStore + 'static>(peer_req: PeeringRequest, ctx: RecvContext<S>) {
    log::debug!("Handling peering request.");

    let peer_salt = peer_req.salt();

    let peer_id = ctx.peer_id.clone();
    let active_peers = ctx.active_peers.clone();
    let request_mngr = ctx.request_mngr.clone();
    let peerstore = ctx.peerstore.clone();
    let server_tx = ctx.server_tx.clone();

    // Note: It is guaranteed, that the spawned future will yield after `RESPONSE_TIMEOUT` milliseconds, so it cannot
    // block the system from shutting down.
    tokio::spawn(async move {
        if let Some(peer) =
            discovery::get_verified_peer(&peer_id, &active_peers, &request_mngr, &peerstore, &server_tx).await
        {
            // request_peering();

            // from := p.disc.GetVerifiedPeer(fromID)
            // status := p.mgr.requestPeering(from, fromSalt)
            // res := newPeeringResponse(rawData, status)

            // // p.logSend(from.Address(), res)
            // s.Send(from.Address(), marshal(res))
        }
    });

    send_peering_response_to_addr(ctx.peer_addr, ctx.peer_id, ctx.msg_bytes, ctx.server_tx, ctx.local);
}

fn handle_peering_response<S: PeerStore>(peer_res: PeeringResponse, peer_reqval: RequestValue<S>, ctx: RecvContext<S>) {
    log::debug!("Handling peering response.");

    // hive.go: PeeringResponse messages are handled in the handleReply function of the validation
}

fn handle_drop_request<S: PeerStore>(_drop_req: DropPeeringRequest, ctx: RecvContext<S>) {
    log::debug!("Handling drop request.");

    let mut removed_nb = ctx.inbound_nbh.write().remove_neighbor(ctx.peer_id);

    if let Some(nb) = ctx.outbound_nbh.write().remove_neighbor(ctx.peer_id) {
        removed_nb.replace(nb);

        ctx.excl_filter.write().exclude_peer(ctx.peer_id.clone());

        // ```go
        //     // if not yet updating, trigger an immediate update
        //     if updateOutResultChan == nil && updateTimer.Stop() {
        //         updateTimer.Reset(0)
        // ```
        todo!("trigger update for outbound neighborhood")
    }

    if removed_nb.is_some() {
        send_drop_peering_request_to_addr(ctx.peer_addr, ctx.peer_id.clone(), ctx.server_tx, ctx.event_tx);
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// SENDING
///////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Initiates a peering request.
///
/// This function is blocking, but at most for `RESPONSE_TIMEOUT` seconds.
pub(crate) async fn begin_peering_request<S: PeerStore>(
    peer_id: &PeerId,
    request_mngr: &RequestManager<S>,
    peerstore: &S,
    server_tx: &ServerTx,
    local: &Local,
) -> Option<Status> {
    let (response_tx, response_rx) = request::response_chan();

    send_peering_request_to_peer(peer_id, request_mngr, peerstore, server_tx, Some(response_tx), local);

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
pub(crate) fn send_peering_request_to_peer<S: PeerStore>(
    peer_id: &PeerId,
    request_mngr: &RequestManager<S>,
    peerstore: &S,
    server_tx: &ServerTx,
    response_tx: Option<ResponseTx>,
    local: &Local,
) {
    let peer = peerstore.fetch_peer(peer_id).expect("peer not in storage");

    let peer_port = peer
        .services()
        .get(AUTOPEERING_SERVICE_NAME)
        .expect("peer doesn't support autopeering")
        .port();

    let peer_addr = SocketAddr::new(peer.ip_address(), peer_port);

    send_peering_request_to_addr(peer_addr, peer_id, request_mngr, server_tx, response_tx, local);
}

/// Sends a peering request to a peer's address.
pub(crate) fn send_peering_request_to_addr<S: PeerStore>(
    peer_addr: SocketAddr,
    peer_id: &PeerId,
    request_mngr: &RequestManager<S>,
    server_tx: &ServerTx,
    response_tx: Option<ResponseTx>,
    local: &Local,
) {
    log::debug!("Sending peering request to: {}", peer_id);

    // Define what happens when we receive the corresponding response.
    let handler = Box::new(|ctx: &RecvContext<S>| {
        // TODO
    });

    let peer_req = request_mngr
        .write()
        .new_peering_request(peer_id.clone(), None, response_tx, local);

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
    local: &Local,
) {
    log::debug!("Sending peering response to: {}", peer_id);

    let request_hash = msg_hash(MessageType::PeeringRequest, msg_bytes).to_vec();

    // TODO: determine status response
    let status = true;

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
pub(crate) fn send_drop_peering_request_to_peer(peer: Peer, server_tx: &ServerTx, event_tx: &EventTx) {
    let peer_port = peer
        .port(AUTOPEERING_SERVICE_NAME)
        .expect("missing autopeering service");
    let peer_addr = SocketAddr::new(peer.ip_address(), peer_port);
    let peer_id = peer.into_id();

    send_drop_peering_request_to_addr(peer_addr, peer_id, server_tx, event_tx);
}

/// Sends a drop-peering request to a peer's address.
pub(crate) fn send_drop_peering_request_to_addr(
    peer_addr: SocketAddr,
    peer_id: PeerId,
    server_tx: &ServerTx,
    event_tx: &EventTx,
) {
    log::debug!("Sending drop request to: {}", peer_id);

    let msg_bytes = DropPeeringRequest::new()
        .to_protobuf()
        .expect("error encoding drop request")
        .to_vec();

    server_tx.send(OutgoingPacket {
        msg_type: MessageType::DropRequest,
        msg_bytes,
        peer_addr,
    });

    publish_drop_peering_event(peer_id, event_tx);
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// EVENTS
///////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Publishes the corresponding peering event [`IncomingPeering`], or [`OutgoingPeering`].
pub(crate) fn publish_peering_event(peer: Peer, dir: Direction, status: Status, local: &Local, event_tx: &EventTx) {
    use Direction::*;

    log::debug!(
        "Peering requested: direction {}, status {}, to {}, #out {}, #in {}",
        dir,
        status,
        peer.peer_id(),
        0,
        0 /* outbound_nbh.num_neighbors(),
           * inbound_nbh.num_neighbors() */
    );

    let distance = salted_distance(
        local.read().peer_id(),
        peer.peer_id(),
        &match dir {
            Inbound => local.read().private_salt().expect("missing private salt").clone(),
            Outbound => local.read().public_salt().expect("missing public salt").clone(),
        },
    );

    event_tx.send(match dir {
        Inbound => Event::IncomingPeering { peer, distance },
        Outbound => Event::OutgoingPeering { peer, distance },
    });
}

fn publish_drop_peering_event(peer_id: PeerId, event_tx: &EventTx) {
    log::debug!(
        "Peering dropped with {}; #out_nbh: {} #in_nbh: {}",
        peer_id,
        0,
        0 /* inbound_nbh.num_neighbors(),
           * outbound_nbh.num_neighbors() */
    );

    event_tx
        .send(Event::PeeringDropped { peer_id })
        .expect("error sending peering-dropped event");
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// HELPERS
///////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub(crate) struct SaltUpdateContext {
    local: Local,
    filter: ExclusionFilter,
    inbound_nbh: InboundNeighborhood,
    outbound_nbh: OutboundNeighborhood,
    server_tx: ServerTx,
    event_tx: EventTx,
}

impl SaltUpdateContext {
    pub(crate) fn new(
        local: Local,
        filter: ExclusionFilter,
        inbound_nbh: InboundNeighborhood,
        outbound_nbh: OutboundNeighborhood,
        server_tx: ServerTx,
        event_tx: EventTx,
    ) -> Self {
        Self {
            local,
            filter,
            inbound_nbh,
            outbound_nbh,
            server_tx,
            event_tx,
        }
    }
}

// Regularly update the salts of the local peer.
pub(crate) fn repeat_update_salts(drop_neighbors_on_salt_update: bool) -> Repeat<SaltUpdateContext> {
    Box::new(move |ctx| {
        update_salts(
            drop_neighbors_on_salt_update,
            &ctx.local,
            &ctx.filter,
            &ctx.inbound_nbh,
            &ctx.outbound_nbh,
            &ctx.server_tx,
            &ctx.event_tx,
        )
    })
}

fn update_salts(
    drop_neighbors_on_salt_update: bool,
    local: &Local,
    filter: &ExclusionFilter,
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

    // Clear the peer filter.
    filter.write().clear_excluded();

    // Either drop, or update the neighborhoods.
    if drop_neighbors_on_salt_update {
        for peer in inbound_nbh.read().iter().chain(outbound_nbh.read().iter()).cloned() {
            send_drop_peering_request_to_peer(peer, server_tx, event_tx);
        }
        inbound_nbh.write().clear();
        outbound_nbh.write().clear();
    } else {
        inbound_nbh.write().update_distances(&local);
        outbound_nbh.write().update_distances(&local);
    }

    log::debug!(
        "Salts updated; private: {}, public: {}",
        private_salt_lifetime,
        public_salt_lifetime,
    );

    // Publish 'SaltUpdated' event.
    event_tx.send(Event::SaltUpdated {
        public_salt_lifetime,
        private_salt_lifetime,
    });
}
