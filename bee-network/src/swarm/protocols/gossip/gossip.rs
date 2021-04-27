// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::GossipHandler;

use crate::network::meta::{ConnectionInfo, Origin};

use libp2p::{
    core::{connection::ConnectionId, ConnectedPoint},
    swarm::{NegotiatedSubstream, NetworkBehaviour, NetworkBehaviourAction, ProtocolsHandler},
    Multiaddr, PeerId,
};
use log::trace;

use std::{
    collections::{HashMap, VecDeque},
    sync::atomic::{AtomicBool, Ordering},
    task::Poll,
};

pub static GOSSIP_ORIGIN: AtomicBool = AtomicBool::new(false);

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
struct Id(PeerId, ConnectionId);

#[derive(Default)]
pub struct Gossip {
    // Gossip event builder per peer id
    builders: HashMap<PeerId, GossipEventBuilder>,
    // Events produced by the 'GossipHandlers'
    events: VecDeque<GossipEvent>,
}

impl Gossip {
    pub fn new() -> Self {
        Self::default()
    }
}

impl NetworkBehaviour for Gossip {
    type ProtocolsHandler = GossipHandler;
    type OutEvent = GossipEvent;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        // FIXME
        let origin = if GOSSIP_ORIGIN.swap(false, Ordering::SeqCst) {
            Origin::Outbound
        } else {
            Origin::Inbound
        };

        trace!("GOSSIP: New_handler: {}", origin);
        GossipHandler::new(origin)
    }

    fn addresses_of_peer(&mut self, peer_id: &libp2p::PeerId) -> Vec<libp2p::Multiaddr> {
        trace!("Addresses of peer: {}", peer_id);
        Vec::new()
    }

    fn inject_connection_established(&mut self, peer_id: &PeerId, conn_id: &ConnectionId, endpoint: &ConnectedPoint) {
        // TODO: Perform the connection checks (not banned, not a duplicate etc)

        let (address, origin) = match endpoint {
            ConnectedPoint::Dialer { address } => (address.clone(), Origin::Outbound),
            ConnectedPoint::Listener { send_back_addr, .. } => (send_back_addr.clone(), Origin::Inbound),
        };

        let builder = GossipEventBuilder::default()
            .with_peer_id(*peer_id)
            .with_peer_addr(address)
            .with_conn_info(ConnectionInfo { id: *conn_id, origin });

        self.builders.insert(*peer_id, builder);
    }

    fn inject_connected(&mut self, _peer_id: &libp2p::PeerId) {}

    fn inject_event(
        &mut self,
        peer_id: PeerId,
        _conn_id: ConnectionId,
        conn: NegotiatedSubstream, //<GossipHandler as ProtocolsHandler>::OutEvent,
    ) {
        if let Some(builder) = self.builders.remove(&peer_id) {
            self.events.push_back(builder.with_conn(conn).finish());
        }
    }

    fn inject_disconnected(&mut self, _peer_id: &libp2p::PeerId) {}

    fn poll(
        &mut self,
        _cx: &mut std::task::Context<'_>,
        _params: &mut impl libp2p::swarm::PollParameters,
    ) -> Poll<NetworkBehaviourAction<<Self::ProtocolsHandler as ProtocolsHandler>::InEvent, Self::OutEvent>> {
        if let Some(event) = self.events.pop_front() {
            Poll::Ready(NetworkBehaviourAction::GenerateEvent(event))
        } else {
            Poll::Pending
        }
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
        // Panic:
        // Unwrapping is fine at this point.
        GossipEvent {
            peer_id: self.peer_id.unwrap(),
            peer_addr: self.peer_addr.unwrap(),
            conn: self.conn.unwrap(),
            conn_info: self.conn_info.unwrap(),
        }
    }
}
