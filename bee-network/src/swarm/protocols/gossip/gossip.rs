use super::GossipHandler;

use crate::host::connections::{ConnectionInfo, Origin};

use futures::{channel::mpsc, SinkExt};
use libp2p::{
    core::{connection::ConnectionId, ConnectedPoint},
    swarm::{NegotiatedSubstream, NetworkBehaviour, NetworkBehaviourAction, ProtocolsHandler},
    Multiaddr, PeerId,
};
use log::*;

use std::{
    collections::{HashMap, VecDeque},
    task::Poll,
};

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
struct Id(PeerId, ConnectionId);

#[derive(Default)]
pub struct Gossip {
    established: HashMap<Id, GossipEventBuilder>,
    completed: VecDeque<GossipEvent>,
    tx: Option<mpsc::Sender<Origin>>,
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
    fn with_peer_id(&mut self, peer_id: PeerId) -> &mut Self {
        self.peer_id.replace(peer_id);
        self
    }

    fn with_peer_addr(&mut self, peer_addr: Multiaddr) -> &mut Self {
        self.peer_addr.replace(peer_addr);
        self
    }

    fn with_conn(&mut self, conn: NegotiatedSubstream) -> &mut Self {
        self.conn.replace(conn);
        self
    }

    fn with_conn_info(&mut self, conn_info: ConnectionInfo) -> &mut Self {
        self.conn_info.replace(conn_info);
        self
    }

    fn is_complete(&self) -> bool {
        self.peer_id.is_some() && self.peer_addr.is_some() && self.conn.is_some() && self.conn_info.is_some()
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
        let (tx, rx) = mpsc::channel::<Origin>(1);
        self.tx.replace(tx);

        GossipHandler::new(rx)
    }

    fn addresses_of_peer(&mut self, peer_id: &libp2p::PeerId) -> Vec<libp2p::Multiaddr> {
        Vec::new()
    }

    fn inject_connection_established(&mut self, peer_id: &PeerId, conn_id: &ConnectionId, endpoint: &ConnectedPoint) {
        let (address, origin) = match endpoint {
            ConnectedPoint::Dialer { address } => (address.clone(), Origin::Outbound),
            ConnectedPoint::Listener { send_back_addr, .. } => (send_back_addr.clone(), Origin::Inbound),
        };

        debug!("Connection ({}) established: {} [{}]", origin, peer_id, address);

        self.established
            .entry(Id(*peer_id, *conn_id))
            .or_default()
            .with_peer_id(*peer_id)
            .with_peer_addr(address)
            .with_conn_info(ConnectionInfo {
                id: *conn_id,
                origin: origin.clone(),
            });

        self.tx
            .as_mut()
            .unwrap()
            .try_send(origin)
            .expect("error sending info about origin to gossip handler");
    }

    fn inject_connected(&mut self, peer_id: &libp2p::PeerId) {}

    fn inject_event(
        &mut self,
        peer_id: PeerId,
        conn_id: ConnectionId,
        conn: <GossipHandler as ProtocolsHandler>::OutEvent,
    ) {
        if let Some(mut builder) = self.established.remove(&Id(peer_id, conn_id)) {
            builder.with_conn(conn);
            debug_assert!(builder.is_complete());
            self.completed.push_back(builder.finish());
        } else {
            return;
        }
    }

    fn inject_disconnected(&mut self, peer_id: &libp2p::PeerId) {}

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
        _params: &mut impl libp2p::swarm::PollParameters,
    ) -> Poll<NetworkBehaviourAction<<Self::ProtocolsHandler as ProtocolsHandler>::InEvent, Self::OutEvent>> {
        if let Some(event) = self.completed.pop_front() {
            Poll::Ready(NetworkBehaviourAction::GenerateEvent(event))
        } else {
            let _ = self.tx.as_mut().unwrap().poll_ready_unpin(cx);
            Poll::Pending
        }
    }
}
