// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    distance::{Neighborhood, SIZE_INBOUND, SIZE_OUTBOUND},
    filter::RejectionFilter,
    messages::{DropRequest, PeeringRequest, PeeringResponse},
};

use crate::{
    command::CommandTx,
    config::AutopeeringConfig,
    event::{Event, EventTx},
    hash,
    local::{
        salt::{self, Salt},
        service_map::AUTOPEERING_SERVICE_NAME,
        Local,
    },
    packet::{msg_hash, IncomingPacket, MessageType, OutgoingPacket},
    peer::{self, peer_id::PeerId, peerstore::PeerStore, Peer},
    request::{self, RequestManager, RequestValue, ResponseTx, RESPONSE_TIMEOUT},
    server::{ServerSocket, ServerTx},
    task::{Runnable, ShutdownRx},
};

use std::net::SocketAddr;

const DEFAULT_OUTBOUND_UPDATE_INTERVAL_SECS: u64 = 1;
const DEFAULT_FULL_OUTBOUND_UPDATE_INTERVAL_SECS: u64 = 60;

type InboundNeighborhood = Neighborhood<SIZE_INBOUND, true>;
type OutboundNeighborhood = Neighborhood<SIZE_OUTBOUND, false>;

pub(crate) type PeeringHandler<S: PeerStore> = Box<dyn Fn(&RecvContext<S>) + Send + Sync + 'static>;

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

pub(crate) struct PeeringManager<S: PeerStore> {
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
    // The storage for discovered peers.
    peerstore: S,
    // Inbound neighborhood.
    inbound_nh: InboundNeighborhood,
    // Outbound neighborhood.
    outbound_nh: OutboundNeighborhood,
    // The peer rejection filter.
    rejection_filter: RejectionFilter,
}

impl<S: PeerStore> PeeringManager<S> {
    pub(crate) fn new(
        config: PeeringManagerConfig,
        local: Local,
        socket: ServerSocket,
        request_mngr: RequestManager<S>,
        peerstore: S,
        event_tx: EventTx,
        command_tx: CommandTx,
    ) -> Self {
        let inbound_nh = Neighborhood::new(local.clone());
        let outbound_nh = Neighborhood::new(local.clone());

        let rejection_filter = RejectionFilter::new();

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
        }
    }
}

#[async_trait::async_trait]
impl<S: PeerStore> Runnable for PeeringManager<S> {
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
            mut inbound_nh,
            mut outbound_nh,
            mut rejection_filter,
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
                            inbound_nbh: &mut inbound_nh,
                            outbound_nbh: &mut outbound_nh,
                            rejection_filter: &mut rejection_filter,
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
                                let drop_req = if let Ok(drop_req) = DropRequest::from_protobuf(&msg_bytes) {
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
    request_mngr: &'a RequestManager<S>,
    peer_addr: SocketAddr,
    event_tx: &'a EventTx,
    inbound_nbh: &'a mut InboundNeighborhood,
    outbound_nbh: &'a mut OutboundNeighborhood,
    rejection_filter: &'a mut RejectionFilter,
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
    } else if salt::is_expired(peer_req.salt_expiration_time()) {
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

    if let Some(reqv) = ctx.request_mngr.pull::<PeeringRequest>(ctx.peer_id) {
        if peer_res.request_hash() != &reqv.request_hash {
            Err(IncorrectRequestHash)
        } else {
            Ok(reqv)
        }
    } else {
        Err(NoCorrespondingRequestOrTimeout)
    }
}

fn validate_drop_request<S: PeerStore>(drop_req: &DropRequest, _: &RecvContext<S>) -> Result<(), ValidationError> {
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

fn handle_peering_request<S: PeerStore>(peer_req: PeeringRequest, ctx: RecvContext<S>) {
    log::debug!("Handling peering request.");

    send_peering_response_to_addr(ctx.peer_addr, ctx.peer_id, ctx.msg_bytes, ctx.server_tx, ctx.local);
}

fn handle_peering_response<S: PeerStore>(peer_res: PeeringResponse, peer_reqval: RequestValue<S>, ctx: RecvContext<S>) {
    log::debug!("Handling peering response.");

    // hive.go: PeeringResponse messages are handled in the handleReply function of the validation
}

fn handle_drop_request<S: PeerStore>(_drop_req: DropRequest, ctx: RecvContext<S>) {
    log::debug!("Handling drop request.");

    let mut removed_peer = ctx.inbound_nbh.remove_peer(ctx.peer_id);

    if let Some(peer) = ctx.outbound_nbh.remove_peer(ctx.peer_id) {
        removed_peer.replace(peer);

        ctx.rejection_filter.include_peer(ctx.peer_id.clone());

        // ```go
        //     // if not yet updating, trigger an immediate update
        //     if updateOutResultChan == nil && updateTimer.Stop() {
        //         updateTimer.Reset(0)
        // ```
    }

    if removed_peer.is_some() {
        send_drop_request_to_addr(ctx.peer_addr, ctx.peer_id, ctx.server_tx);
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// SENDING
///////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) async fn begin_peering_request<S: PeerStore>(
    peer_id: &PeerId,
    request_mngr: &RequestManager<S>,
    peerstore: &S,
    server_tx: &ServerTx,
) -> Option<bool> {
    let (response_tx, response_rx) = request::response_chan();

    send_peering_request_to_peer(peer_id, request_mngr, peerstore, server_tx, Some(response_tx));

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

pub(crate) fn send_peering_request_to_peer<S: PeerStore>(
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

    send_peering_request_to_addr(peer_addr, peer_id, request_mngr, server_tx, response_tx);
}

pub(crate) fn send_peering_request_to_addr<S: PeerStore>(
    peer_addr: SocketAddr,
    peer_id: &PeerId,
    request_mngr: &RequestManager<S>,
    server_tx: &ServerTx,
    response_tx: Option<ResponseTx>,
) {
    log::debug!("Sending peering request to: {}", peer_id);

    // Define what happens when we receive the corresponding response.
    let handler = Box::new(|ctx: &RecvContext<S>| {
        // TODO
    });

    let peer_req = request_mngr.new_peering_request(peer_id.clone(), None, response_tx);

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

// Sends a drop request to a peer.
fn send_drop_request_to_peer<S: PeerStore>(peer_id: &PeerId, peerstore: &S, server_tx: &ServerTx) {
    let peer = peerstore.fetch_peer(peer_id).expect("peer not in storage");

    let peer_port = peer
        .services()
        .get(AUTOPEERING_SERVICE_NAME)
        .expect("invalid autopeering peer")
        .port();

    let peer_addr = SocketAddr::new(peer.ip_address(), peer_port);

    send_drop_request_to_addr(peer_addr, peer_id, server_tx);
}

// Sends a drop request to a peer's address.
pub(crate) fn send_drop_request_to_addr(peer_addr: SocketAddr, peer_id: &PeerId, server_tx: &ServerTx) {
    log::debug!("Sending drop request to: {}", peer_id);

    let msg_bytes = DropRequest::new()
        .to_protobuf()
        .expect("error encoding drop request")
        .to_vec();

    server_tx.send(OutgoingPacket {
        msg_type: MessageType::DropRequest,
        msg_bytes,
        peer_addr,
    });
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// HELPERS
///////////////////////////////////////////////////////////////////////////////////////////////////////////

fn update_salts(
    local: &Local,
    filter: &mut RejectionFilter,
    drop_neighbors_on_salt_update: bool,
    inbound: &mut InboundNeighborhood,
    outbound: &mut OutboundNeighborhood,
    packet_tx: &ServerTx,
    event_tx: &EventTx,
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
    event_tx.send(Event::SaltUpdated);
}

fn drop_neighborhood<'a, H>(neighborhood: &'a H, server_tx: &ServerTx)
where
    &'a H: IntoIterator<Item = Peer, IntoIter = std::vec::IntoIter<Peer>>,
{
    for peer in neighborhood {
        let port = peer
            .services()
            .get(AUTOPEERING_SERVICE_NAME)
            .expect("missing autopeering service")
            .port();

        let peer_addr = SocketAddr::new(peer.ip_address(), port);

        send_drop_request_to_addr(peer_addr, peer.peer_id(), server_tx);
    }
}
