// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::GossipHandler;

use crate::host::{ConnectionInfo, Origin};

use libp2p::{
    core::{connection::ConnectionId, ConnectedPoint},
    swarm::{NegotiatedSubstream, NetworkBehaviour, NetworkBehaviourAction, ProtocolsHandler},
    Multiaddr, PeerId,
};
use log::*;
use tokio::sync::mpsc;

use std::task::Poll;

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
struct Id(PeerId, ConnectionId);

#[derive(Default)]
pub struct Gossip {
    builder: Option<GossipEventBuilder>,
    event: Option<GossipEvent>,
    origin_tx: Option<mpsc::Sender<Origin>>,
}

impl Gossip {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct GossipEvent {
    pub peer_id: PeerId,
    pub peer_addr: Multiaddr,
    pub conn: NegotiatedSubstream,
    pub conn_info: ConnectionInfo,
}

#[derive(Default)]
struct GossipEventBuilder {
    peer_id: Option<PeerId>,
    peer_addr: Option<Multiaddr>,
    conn: Option<NegotiatedSubstream>,
    conn_info: Option<ConnectionInfo>,
}

impl GossipEventBuilder {
    fn with_peer_id(mut self, peer_id: PeerId) -> Self {
        self.peer_id.replace(peer_id);
        self
    }

    fn with_peer_addr(mut self, peer_addr: Multiaddr) -> Self {
        self.peer_addr.replace(peer_addr);
        self
    }

    fn with_conn(mut self, conn: NegotiatedSubstream) -> Self {
        self.conn.replace(conn);
        self
    }

    fn with_conn_info(mut self, conn_info: ConnectionInfo) -> Self {
        self.conn_info.replace(conn_info);
        self
    }

    fn finish(self) -> GossipEvent {
        // NOTE: calling 'finish' without prior checking with 'is_complete' is considered a programmer error,
        // hence 'unwrap' is safe.
        GossipEvent {
            peer_id: self.peer_id.unwrap(),
            peer_addr: self.peer_addr.unwrap(),
            conn: self.conn.unwrap(),
            conn_info: self.conn_info.unwrap(),
        }
    }
}

impl NetworkBehaviour for Gossip {
    type ProtocolsHandler = GossipHandler;
    type OutEvent = GossipEvent; //<Self::ProtocolsHandler as ProtocolsHandler>::OutEvent;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        let (origin_tx, origin_rx) = mpsc::channel::<Origin>(1);
        self.origin_tx.replace(origin_tx);
        GossipHandler::new(origin_rx)
    }

    fn addresses_of_peer(&mut self, _peer_id: &libp2p::PeerId) -> Vec<libp2p::Multiaddr> {
        Vec::new()
    }

    fn inject_connection_established(&mut self, peer_id: &PeerId, conn_id: &ConnectionId, endpoint: &ConnectedPoint) {
        let (address, origin) = match endpoint {
            ConnectedPoint::Dialer { address } => (address.clone(), Origin::Outbound),
            ConnectedPoint::Listener { send_back_addr, .. } => (send_back_addr.clone(), Origin::Inbound),
        };

        trace!("GOSSIP: Connection ({}) established: {} [{}]", origin, peer_id, address);

        let builder = GossipEventBuilder::default()
            .with_peer_id(*peer_id)
            .with_peer_addr(address)
            .with_conn_info(ConnectionInfo { id: *conn_id, origin });

        self.builder.replace(builder);

        if let Some(origin_tx) = self.origin_tx.take() {
            origin_tx
                .try_send(origin)
                .expect("error propagation origin to gossip handler");
        }
    }

    fn inject_connected(&mut self, _peer_id: &libp2p::PeerId) {}

    fn inject_event(
        &mut self,
        _peer_id: PeerId,
        _conn_id: ConnectionId,
        conn: NegotiatedSubstream, //<GossipHandler as ProtocolsHandler>::OutEvent,
    ) {
        trace!("GOSSIP: EVENT");

        if let Some(builder) = self.builder.take() {
            trace!("GOSSIP: FINISH BUILDER");
            self.event.replace(builder.with_conn(conn).finish());
        }
    }

    fn inject_disconnected(&mut self, _peer_id: &libp2p::PeerId) {}

    fn poll(
        &mut self,
        _cx: &mut std::task::Context<'_>,
        _params: &mut impl libp2p::swarm::PollParameters,
    ) -> Poll<NetworkBehaviourAction<<Self::ProtocolsHandler as ProtocolsHandler>::InEvent, Self::OutEvent>> {
        if let Some(event) = self.event.take() {
            trace!("GOSSIP: POLL READY");
            Poll::Ready(NetworkBehaviourAction::GenerateEvent(event))
        } else {
            trace!("GOSSIP: POLL PENDING");
            Poll::Pending
        }
    }
}
