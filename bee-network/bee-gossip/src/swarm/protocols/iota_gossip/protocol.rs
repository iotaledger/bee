// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    event::{IotaGossipEvent, IotaGossipHandlerEvent},
    handler::{GossipProtocolHandler, IotaGossipHandlerInEvent},
    id::IotaGossipIdentifier,
};

use crate::{alias, init::global::network_id, network::origin::Origin};

use libp2p::{
    core::{connection::ConnectionId, ConnectedPoint},
    swarm::{NetworkBehaviour, NetworkBehaviourAction, NotifyHandler, PollParameters},
    Multiaddr, PeerId,
};
use log::debug;

use std::{
    collections::{HashMap, VecDeque},
    task::{Context, Poll},
};

const IOTA_GOSSIP_NAME: &str = "iota-gossip";
const IOTA_GOSSIP_VERSION: &str = "1.0.0";

struct ConnectionInfo {
    addr: Multiaddr,
    origin: Origin,
}

/// Substream upgrade protocol for `/iota-gossip/1.0.0`.
pub struct IotaGossipProtocol {
    /// The gossip protocol identifier.
    id: IotaGossipIdentifier,

    /// Counts the number of handlers created.
    num_handlers: usize,

    /// Counts the number of inbound connections.
    num_inbounds: usize,

    /// Counts the number of outbound connections.
    num_outbounds: usize,

    /// Events produced for the behavior and handlers.
    events: VecDeque<NetworkBehaviourAction<IotaGossipHandlerInEvent, IotaGossipEvent>>,

    /// Maps peers to their connection infos. Peers can only have 1 gossip connection, hence the mapping is 1:1.
    peers: HashMap<PeerId, ConnectionInfo>,
}

impl IotaGossipProtocol {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for IotaGossipProtocol {
    fn default() -> Self {
        Self {
            id: IotaGossipIdentifier::new(IOTA_GOSSIP_NAME, network_id(), IOTA_GOSSIP_VERSION),
            num_handlers: 0,
            num_inbounds: 0,
            num_outbounds: 0,
            events: VecDeque::with_capacity(16),
            peers: HashMap::with_capacity(8),
        }
    }
}

impl NetworkBehaviour for IotaGossipProtocol {
    type ProtocolsHandler = GossipProtocolHandler;
    type OutEvent = IotaGossipEvent;

    /// **libp2p docs**:
    ///
    /// Creates a new `ProtocolsHandler` for a connection with a peer.
    ///
    /// Every time an incoming connection is opened, and every time we start dialing a node, this
    /// method is called.
    ///
    /// The returned object is a handler for that specific connection, and will be moved to a
    /// background task dedicated to that connection.
    ///
    /// The network behaviour (ie. the implementation of this trait) and the handlers it has
    /// spawned (ie. the objects returned by `new_handler`) can communicate by passing messages.
    /// Messages sent from the handler to the behaviour are injected with `inject_event`, and
    /// the behaviour can send a message to the handler by making `poll` return `SendEvent`.
    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        self.num_handlers += 1;
        debug!("gossip protocol: new handler ({}).", self.num_handlers);

        GossipProtocolHandler::new(self.id.clone())
    }

    /// **libp2p docs**:
    ///
    /// Addresses that this behaviour is aware of for this specific peer, and that may allow
    /// reaching the peer.
    ///
    /// The addresses will be tried in the order returned by this function, which means that they
    /// should be ordered by decreasing likelihood of reachability. In other words, the first
    /// address should be the most likely to be reachable.
    fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        let addrs = self
            .peers
            .get(peer_id)
            .map_or(Vec::new(), |conn_info| vec![conn_info.addr.clone()]);

        debug!("gossip protocol: addresses of peer {}: {:?}.", alias!(peer_id), addrs);

        addrs
    }

    /// **libp2p docs**:
    ///
    /// Informs the behaviour about a newly established connection to a peer.
    fn inject_connection_established(&mut self, peer_id: &PeerId, conn_id: &ConnectionId, endpoint: &ConnectedPoint) {
        let (peer_addr, origin) = match endpoint {
            ConnectedPoint::Dialer { address } => (address.clone(), Origin::Outbound),
            ConnectedPoint::Listener { send_back_addr, .. } => (send_back_addr.clone(), Origin::Inbound),
        };

        match origin {
            Origin::Inbound => self.num_inbounds += 1,
            Origin::Outbound => self.num_outbounds += 1,
        }
        debug!(
            "gossip protocol: connection established: inbound/outbound: {}/{}",
            self.num_inbounds, self.num_outbounds
        );

        self.peers.insert(*peer_id, {
            ConnectionInfo {
                addr: peer_addr,
                origin,
            }
        });

        let handler_event = IotaGossipHandlerInEvent { origin };

        let notify_handler = NetworkBehaviourAction::NotifyHandler {
            peer_id: *peer_id,
            handler: NotifyHandler::One(*conn_id), // TODO: maybe better use ::Any ??
            event: handler_event,
        };

        self.events.push_back(notify_handler);
    }

    /// **libp2p docs**:
    ///
    /// Indicate to the behaviour that we connected to the node with the given peer id.
    ///
    /// This node now has a handler (as spawned by `new_handler`) running in the background.
    ///
    /// This method is only called when the first connection to the peer is established, preceded by
    /// [`inject_connection_established`](NetworkBehaviour::inject_connection_established).
    fn inject_connected(&mut self, peer_id: &PeerId) {
        debug!("gossip protocol: {} connected.", alias!(peer_id));
    }

    /// **libp2p docs**:
    ///
    /// Informs the behaviour about an event generated by the handler dedicated to the peer identified by `peer_id`.
    /// for the behaviour.
    ///
    /// The `peer_id` is guaranteed to be in a connected state. In other words, `inject_connected`
    /// has previously been called with this `PeerId`.
    fn inject_event(&mut self, peer_id: PeerId, _: ConnectionId, event: IotaGossipHandlerEvent) {
        debug!("gossip protocol: handler event: {:?}", event);

        // Propagate events to the behavior.
        let ev = match event {
            IotaGossipHandlerEvent::SentUpgradeRequest { to } => {
                NetworkBehaviourAction::GenerateEvent(IotaGossipEvent::SentUpgradeRequest { to })
            }
            IotaGossipHandlerEvent::UpgradeCompleted { substream } => {
                if let Some(conn_info) = self.peers.remove(&peer_id) {
                    NetworkBehaviourAction::GenerateEvent(IotaGossipEvent::UpgradeCompleted {
                        peer_id,
                        peer_addr: conn_info.addr,
                        origin: conn_info.origin,
                        substream,
                    })
                } else {
                    return;
                }
            }
            IotaGossipHandlerEvent::UpgradeError { peer_id, error } => {
                NetworkBehaviourAction::GenerateEvent(IotaGossipEvent::UpgradeError { peer_id, error })
            }
            _ => return,
        };

        self.events.push_back(ev);
    }

    /// **libp2p docs**:
    ///
    /// Informs the behaviour about a closed connection to a peer.
    ///
    /// A call to this method is always paired with an earlier call to
    /// `inject_connection_established` with the same peer ID, connection ID and
    /// endpoint.
    fn inject_connection_closed(&mut self, peer_id: &PeerId, _: &ConnectionId, _: &ConnectedPoint) {
        debug!("gossip behavior: connection with {} closed.", alias!(peer_id));
    }

    /// **libp2p docs**:
    ///
    /// Indicates to the behaviour that we disconnected from the node with the given peer id.
    ///
    /// There is no handler running anymore for this node. Any event that has been sent to it may
    /// or may not have been processed by the handler.
    ///
    /// This method is only called when the last established connection to the peer is closed,
    /// preceded by [`inject_connection_closed`](NetworkBehaviour::inject_connection_closed).
    fn inject_disconnected(&mut self, peer_id: &PeerId) {
        debug!("gossip behavior: {} disconnected.", alias!(peer_id));
    }

    /// **libp2p docs**:
    ///
    /// Informs the behaviour that the [`ConnectedPoint`] of an existing connection has changed.
    fn inject_address_change(
        &mut self,
        peer_id: &PeerId,
        _: &ConnectionId,
        _old: &ConnectedPoint,
        _new: &ConnectedPoint,
    ) {
        debug!("gossip behavior: address of {} changed.", alias!(peer_id));
    }

    fn poll(
        &mut self,
        _: &mut Context<'_>,
        _: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<IotaGossipHandlerInEvent, Self::OutEvent>> {
        if let Some(event) = self.events.pop_front() {
            Poll::Ready(event)
        } else {
            Poll::Pending
        }
    }
}
