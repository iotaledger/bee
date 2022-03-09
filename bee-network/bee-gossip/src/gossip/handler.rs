// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{behaviour::GossipHandlerCommand, protocol::GossipProtocol};

use libp2p::{
    swarm::{
        protocols_handler::{
            KeepAlive, ProtocolsHandler, ProtocolsHandlerEvent, ProtocolsHandlerUpgrErr, SubstreamProtocol,
        },
        NegotiatedSubstream,
    },
    Multiaddr, PeerId,
};

use std::{
    collections::VecDeque,
    io,
    task::{Context, Poll},
};

type GossipProtocolHandlerEvent = ProtocolsHandlerEvent<GossipProtocol, (), GossipHandlerEvent, io::Error>;
type GossipSubstreamProtocol = SubstreamProtocol<GossipProtocol, ()>;

#[derive(Debug)]
pub(crate) enum GossipHandlerEvent {
    ProtocolEstablished {
        peer_addr: Multiaddr,
        substream: NegotiatedSubstream,
    },
    // FIXME
    #[allow(dead_code)]
    ProtocolNegotiationError {
        peer_id: PeerId,
        error: ProtocolsHandlerUpgrErr<io::Error>,
    },
}

pub(crate) struct GossipHandler {
    index: usize,
    network_name: &'static str,
    events: VecDeque<GossipProtocolHandlerEvent>,
    peer_addr: Option<Multiaddr>,
}

impl GossipHandler {
    pub(crate) fn new(index: usize, network_name: &'static str) -> Self {
        Self {
            index,
            network_name,
            events: VecDeque::default(),
            peer_addr: None,
        }
    }
}

impl ProtocolsHandler for GossipHandler {
    type InEvent = GossipHandlerCommand;
    type OutEvent = GossipHandlerEvent;
    type Error = io::Error;
    type InboundProtocol = GossipProtocol;
    type OutboundProtocol = GossipProtocol;
    type InboundOpenInfo = ();
    type OutboundOpenInfo = ();

    /// Tries to make progress with the handler events.
    fn poll(&mut self, _: &mut Context<'_>) -> Poll<GossipProtocolHandlerEvent> {
        if let Some(event) = self.events.pop_front() {
            log::trace!("Handler event ready: {event:?}");
            Poll::Ready(event)
        } else {
            log::trace!("Waiting for next handler event.");
            Poll::Pending
        }
    }

    /// Used to construct a `GossipProtocol` instance for the listener.
    fn listen_protocol(&self) -> GossipSubstreamProtocol {
        log::trace!("#{}: Requested substream/gossip protocol.", self.index);

        new_gossip_substream_protocol(self.network_name)
    }

    /// Executes whenever the protocol behaviour sends a  `NetworkBehaviourAction::NotifyHandler` action.
    ///
    /// Note:
    /// We use it to send the upgrade request iff we are the dialer. The internal `libp2p` machinery will process
    /// this event, and handle the request/response cycle.
    fn inject_event(&mut self, event: GossipHandlerCommand) {
        log::trace!("#{}: Received gossip handler command.", self.index);

        match event {
            GossipHandlerCommand::KeepPeerAddr(peer_addr) => {
                self.peer_addr.replace(peer_addr);
            }
            GossipHandlerCommand::SendUpgradeRequest => {
                let send_upgrade_request_event = ProtocolsHandlerEvent::OutboundSubstreamRequest {
                    protocol: new_gossip_substream_protocol(self.network_name),
                };

                self.events.push_back(send_upgrade_request_event);
            }
        }
    }

    /// Executes when the gossip protocol has been successfully negotiated on an inbound connection.
    ///
    /// Note:
    /// The generated custom event will be handled in the `inject_event` method of the gossip behaviour.
    fn inject_fully_negotiated_inbound(&mut self, substream: NegotiatedSubstream, _: ()) {
        log::trace!("#{}: Inbound upgrade successful.", self.index);

        let peer_addr = self.peer_addr.take().expect("take peer addr");

        let inbound_upgrade_successful_event =
            ProtocolsHandlerEvent::Custom(GossipHandlerEvent::ProtocolEstablished { peer_addr, substream });

        self.events.push_back(inbound_upgrade_successful_event);
    }

    /// Executes when the gossip protocol has been successfully negotiated on an outbound connection.
    ///
    /// Note:
    /// The generated custom event will be handled in the `inject_event` method of the gossip behaviour.
    fn inject_fully_negotiated_outbound(&mut self, substream: NegotiatedSubstream, _: ()) {
        log::trace!("#{}: Outbound upgrade successful.", self.index);

        let peer_addr = self.peer_addr.take().expect("take peer addr");

        let outbound_upgrade_successful_event =
            ProtocolsHandlerEvent::Custom(GossipHandlerEvent::ProtocolEstablished { peer_addr, substream });

        self.events.push_back(outbound_upgrade_successful_event);
    }

    fn inject_dial_upgrade_error(&mut self, _: (), _: ProtocolsHandlerUpgrErr<io::Error>) {
        log::trace!("#{}: Dial upgrade error.", self.index);
    }

    fn connection_keep_alive(&self) -> KeepAlive {
        KeepAlive::Yes
    }

    // Default members

    fn inject_address_change(&mut self, _new_address: &Multiaddr) {}

    fn inject_listen_upgrade_error(
        &mut self,
        _: Self::InboundOpenInfo,
        _: ProtocolsHandlerUpgrErr<<Self::InboundProtocol as libp2p::swarm::protocols_handler::InboundUpgradeSend>::Error>,
    ) {
    }
}

impl Drop for GossipHandler {
    fn drop(&mut self) {
        log::trace!("Handler #{} dropped.", self.index);
    }
}

fn new_gossip_substream_protocol(network_name: &'static str) -> GossipSubstreamProtocol {
    SubstreamProtocol::new(GossipProtocol::new(network_name), ())
}

// #[derive(Debug)]
// pub(crate) enum GossipHandlerEvent {
//     /// Waiting for an upgrade request when inbound.
//     AwaitingUpgradeRequest { from: PeerId },

//     /// Received request for IOTA gossip protocol upgrade.
//     ReceivedUpgradeRequest { from: PeerId },

//     /// Sent request for IOTA gossip protocol upgrade.
//     SentUpgradeRequest { to: PeerId },

//     /// Successfully upgraded to the IOTA gossip protocol.
//     UpgradeCompleted { substream: Box<NegotiatedSubstream> },

//     /// An errror occured during the upgrade.
//     UpgradeError {
//         peer_id: PeerId,
//         error: ProtocolsHandlerUpgrErr<io::Error>,
//     },
// }

// #[derive(Debug)]
// pub struct GossipHandlerInEvent {
//     pub origin: Origin,
// }

// impl ProtocolsHandler for GossipProtocolHandler {
//     type InEvent = GossipHandlerInEvent;
//     type OutEvent = GossipHandlerEvent;
//     type Error = io::Error;
//     type InboundProtocol = GossipProtocol;
//     type OutboundProtocol = GossipProtocol;
//     type InboundOpenInfo = ();
//     type OutboundOpenInfo = ();

//     fn poll(&mut self, _: &mut Context<'_>) -> Poll<GossipProtocolHandlerEvent> {
//         if let Some(event) = self.events.pop_front() {
//             Poll::Ready(event)
//         } else {
//             Poll::Pending
//         }
//     }

//     fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
//         debug!("gossip handler: responding to listen protocol request.");

//         SubstreamProtocol::new(GossipProtocol::new(self.info.clone()), ())
//     }

//     fn inject_event(&mut self, incoming_event: GossipHandlerInEvent) {
//         debug!("gossip handler: received in-event: {:?}", incoming_event);

//         let GossipHandlerInEvent { origin } = incoming_event;

//         // We only send the upgrade request if this handler belongs to an outbound connection.
//         if origin == Origin::Outbound {
//             let send_request = ProtocolsHandlerEvent::OutboundSubstreamRequest {
//                 protocol: SubstreamProtocol::new(GossipProtocol::new(self.info.clone()), ()),
//             };

//             debug!("gossip handler: sending protocol upgrade request.");

//             self.events.push_back(send_request);
//         }
//     }

//     fn inject_fully_negotiated_inbound(&mut self, new_inbound: NegotiatedSubstream, _: Self::InboundOpenInfo) {
//         let negotiated_inbound = ProtocolsHandlerEvent::Custom(GossipHandlerEvent::UpgradeCompleted {
//             substream: Box::new(new_inbound),
//         });

//         debug!("gossip handler: fully negotiated inbound.");

//         self.events.push_back(negotiated_inbound);
//     }

//     fn inject_fully_negotiated_outbound(&mut self, new_outbound: NegotiatedSubstream, _: Self::OutboundOpenInfo) {
//         let negotiated_outbound = ProtocolsHandlerEvent::Custom(GossipHandlerEvent::UpgradeCompleted {
//             substream: Box::new(new_outbound),
//         });

//         debug!("gossip handler: fully negotiated outbound.");

//         self.events.push_back(negotiated_outbound);
//     }

//     fn inject_address_change(&mut self, new_address: &Multiaddr) {
//         debug!("gossip handler: new address: {}", new_address);
//     }

//     fn inject_dial_upgrade_error(
//         &mut self,
//         _: Self::OutboundOpenInfo,
//         e: ProtocolsHandlerUpgrErr<<Self::OutboundProtocol as OutboundUpgrade<NegotiatedSubstream>>::Error>,
//     ) {
//         debug!("gossip handler: outbound upgrade error: {:?}", e);

//         // TODO: finish event management in case of an error.
//         // self.events.push_back(ProtocolsHandlerEvent::Close(e));
//     }

//     fn inject_listen_upgrade_error(
//         &mut self,
//         _: Self::InboundOpenInfo,
//         e: ProtocolsHandlerUpgrErr<<Self::InboundProtocol as InboundUpgradeSend>::Error>,
//     ) {
//         debug!("gossip handler: inbound upgrade error: {:?}", e);

//         // TODO: finish event management in case of an error.
//         // let err = match e {
//         //     ProtocolsHandlerUpgrErr::Timeout => io::Error::new(io::ErrorKind::TimedOut, "timeout"),
//         //     ProtocolsHandlerUpgrErr::Timer => io::Error::new(io::ErrorKind::TimedOut, "timer"),
//         //     ProtocolsHandlerUpgrErr::Upgrade(err) => err,
//         // };

//         // self.events.push_back(ProtocolsHandlerEvent::Close(err));
//     }

//     fn connection_keep_alive(&self) -> KeepAlive {
//         self.keep_alive
//     }
// }
