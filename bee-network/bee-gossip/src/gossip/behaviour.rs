// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::handler::{GossipHandler, GossipHandlerEvent};

use libp2p::{
    core::{
        connection::{ConnectionId, ListenerId},
        ConnectedPoint,
    },
    swarm::{
        DialError, IntoProtocolsHandler, NegotiatedSubstream, NetworkBehaviour, NetworkBehaviourAction, NotifyHandler,
        PollParameters, ProtocolsHandlerUpgrErr,
    },
    Multiaddr, PeerId,
};

use std::{
    collections::VecDeque,
    io,
    task::{Context, Poll},
};

/// A type alias tailored to the needs of the gossip behaviour.
type GossipBehaviourAction = NetworkBehaviourAction<GossipEvent, GossipHandler, GossipHandlerCommand>;

#[derive(Debug)]
pub(crate) enum GossipHandlerCommand {
    KeepPeerAddr(Multiaddr),
    SendUpgradeRequest,
}

/// Events produces by the gossip behaviour.
#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum GossipEvent {
    Established {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        substream: NegotiatedSubstream,
    },
    NegotiationError {
        peer_id: PeerId,
        // FIXME:
        #[allow(dead_code)]
        error: ProtocolsHandlerUpgrErr<io::Error>,
    },
    // FIXME
    #[allow(dead_code)]
    Terminated { peer_id: PeerId },
}

/// A glue type between the gossip layer and the gossip handlers created for each peer respectively.
pub(crate) struct Gossip {
    network_name: &'static str,
    num_created_handlers: usize,
    actions: VecDeque<GossipBehaviourAction>,
}

impl Gossip {
    pub(crate) fn new(network_name: &'static str) -> Self {
        Self {
            network_name,
            num_created_handlers: 0,
            actions: VecDeque::default(),
        }
    }
}

impl NetworkBehaviour for Gossip {
    type ProtocolsHandler = GossipHandler;
    type OutEvent = GossipEvent;

    fn poll(&mut self, _: &mut Context<'_>, _: &mut impl PollParameters) -> Poll<GossipBehaviourAction> {
        if let Some(action) = self.actions.pop_front() {
            log::trace!("Behaviour action ready.");
            Poll::Ready(action)
        } else {
            log::trace!("Waiting for next behaviour action");
            Poll::Pending
        }
    }

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        let handler_index = self.num_created_handlers;
        self.num_created_handlers += 1;

        log::trace!("Requested new protocol handler: created #{handler_index}");

        GossipHandler::new(handler_index, self.network_name)
    }

    fn inject_connection_established(
        &mut self,
        peer_id: &PeerId,
        conn_id: &ConnectionId,
        endpoint: &ConnectedPoint,
        _failed_addresses: Option<&Vec<Multiaddr>>,
    ) {
        log::trace!("Connection established with peer: {peer_id}, conn: {conn_id:?}, endpoint: {endpoint:?}");

        let peer_addr = match endpoint {
            ConnectedPoint::Dialer { address } => address.clone(),
            ConnectedPoint::Listener { send_back_addr, .. } => send_back_addr.clone(),
        };

        let keep_peer_addr_action = GossipBehaviourAction::NotifyHandler {
            peer_id: *peer_id,
            handler: NotifyHandler::One(*conn_id),
            event: GossipHandlerCommand::KeepPeerAddr(peer_addr),
        };

        self.actions.push_back(keep_peer_addr_action);

        if endpoint.is_dialer() {
            let upgrade_request_action = GossipBehaviourAction::NotifyHandler {
                peer_id: *peer_id,
                handler: NotifyHandler::One(*conn_id),
                event: GossipHandlerCommand::SendUpgradeRequest,
            };

            self.actions.push_back(upgrade_request_action);
        }
    }

    /// Intercepts events produced by the protocol handler.
    fn inject_event(&mut self, peer_id: PeerId, conn_id: ConnectionId, handler_event: GossipHandlerEvent) {
        log::trace!("Handler event for peer: {peer_id}, conn: {conn_id:?}, event: {handler_event:?}",);

        let behaviour_event = match handler_event {
            GossipHandlerEvent::ProtocolEstablished { peer_addr, substream } => GossipEvent::Established {
                peer_id,
                peer_addr,
                substream,
            },
            GossipHandlerEvent::ProtocolNegotiationError { peer_id, error } => {
                GossipEvent::NegotiationError { peer_id, error }
            }
        };

        let behaviour_action = GossipBehaviourAction::GenerateEvent(behaviour_event);

        self.actions.push_back(behaviour_action);
    }

    fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        log::trace!("Addresses requested for peer: {}", peer_id);

        vec![]
    }

    fn inject_connected(&mut self, peer_id: &PeerId) {
        log::trace!("Connected peer: {peer_id}");
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId) {
        log::trace!("Disconnected peer: {peer_id}");

        // TODO: publish GossipEvent::ProtocolTerminated
    }

    fn inject_connection_closed(
        &mut self,
        peer_id: &PeerId,
        conn_id: &ConnectionId,
        endpoint: &ConnectedPoint,
        _: <Self::ProtocolsHandler as IntoProtocolsHandler>::Handler,
    ) {
        log::trace!("Connection closed with peer: {peer_id}, conn: {conn_id:?}, point: {endpoint:?}");

        // TODO: publish GossipEvent::ProtocolTerminated
    }

    fn inject_address_change(&mut self, _: &PeerId, _: &ConnectionId, _old: &ConnectedPoint, _new: &ConnectedPoint) {}

    fn inject_dial_failure(&mut self, peer_id: Option<PeerId>, _handler: Self::ProtocolsHandler, error: &DialError) {
        log::trace!("Failed to dial peer: {peer_id:?}. Cause: {error}")
    }

    fn inject_listen_failure(
        &mut self,
        local_addr: &libp2p::Multiaddr,
        send_back_addr: &libp2p::Multiaddr,
        _: Self::ProtocolsHandler,
    ) {
        log::trace!("Failed to listen on: {local_addr} from {send_back_addr}.");
    }

    fn inject_new_listener(&mut self, id: ListenerId) {
        log::trace!("New listener #{id:?}");
    }

    fn inject_new_listen_addr(&mut self, id: ListenerId, addr: &Multiaddr) {
        log::trace!("New listen addr {addr} for listener {id:?}");
    }

    fn inject_expired_listen_addr(&mut self, id: ListenerId, addr: &Multiaddr) {
        log::trace!("Expired listen addr {addr} for listener {id:?}");
    }

    fn inject_listener_error(&mut self, id: ListenerId, err: &(dyn std::error::Error + 'static)) {
        log::trace!("Listener error {err} for listener {id:?}");
    }

    fn inject_listener_closed(&mut self, id: ListenerId, reason: Result<(), &std::io::Error>) {
        log::trace!("Listener closed #{id:?}. Reason: {reason:?}");
    }

    fn inject_new_external_addr(&mut self, addr: &Multiaddr) {
        log::trace!("New external addr {addr}");
    }

    fn inject_expired_external_addr(&mut self, addr: &Multiaddr) {
        log::trace!("Expired external addr {addr}");
    }
}
