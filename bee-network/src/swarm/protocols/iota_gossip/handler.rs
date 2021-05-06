// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{event::IotaGossipHandlerEvent, id::IotaGossipIdentifier, upgrade::IotaGossipProtocolUpgrade};

use crate::network::origin::Origin;

use libp2p::{
    core::upgrade::OutboundUpgrade,
    swarm::{
        protocols_handler::{
            InboundUpgradeSend, KeepAlive, ProtocolsHandler, ProtocolsHandlerEvent, ProtocolsHandlerUpgrErr,
            SubstreamProtocol,
        },
        NegotiatedSubstream,
    },
    Multiaddr,
};

use log::*;

use std::{
    collections::VecDeque,
    io,
    task::{Context, Poll},
};

pub struct GossipProtocolHandler {
    /// Exchanged protocol information necessary during negotiation.
    info: IotaGossipIdentifier,

    /// Keep alive setting.
    keep_alive: KeepAlive,

    /// All events produced by this handler.
    events: VecDeque<ProtocolsHandlerEvent<IotaGossipProtocolUpgrade, (), IotaGossipHandlerEvent, io::Error>>,
}

#[derive(Debug)]
pub struct IotaGossipHandlerInEvent {
    pub origin: Origin,
}

impl GossipProtocolHandler {
    pub fn new(info: IotaGossipIdentifier) -> Self {
        Self {
            info,
            keep_alive: KeepAlive::Yes,
            events: VecDeque::with_capacity(16),
        }
    }
}

impl ProtocolsHandler for GossipProtocolHandler {
    type InEvent = IotaGossipHandlerInEvent;
    type OutEvent = IotaGossipHandlerEvent;
    type Error = io::Error;
    type InboundProtocol = IotaGossipProtocolUpgrade;
    type OutboundProtocol = IotaGossipProtocolUpgrade;
    type InboundOpenInfo = ();
    type OutboundOpenInfo = ();

    /// **libp2p docs**:
    ///
    /// The [`InboundUpgrade`](libp2p_core::upgrade::InboundUpgrade) to apply on inbound
    /// substreams to negotiate the desired protocols.
    ///
    /// > **Note**: The returned `InboundUpgrade` should always accept all the generally
    /// >           supported protocols, even if in a specific context a particular one is
    /// >           not supported, (eg. when only allowing one substream at a time for a protocol).
    /// >           This allows a remote to put the list of supported protocols in a cache.
    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
        debug!("gossip handler: responding to listen protocol request.");

        SubstreamProtocol::new(IotaGossipProtocolUpgrade::new(self.info.clone()), ())
    }

    /// **libp2p docs**:
    ///
    /// Injects an event coming from the outside in the handler.
    fn inject_event(&mut self, incoming_event: IotaGossipHandlerInEvent) {
        debug!("gossip handler: received in-event: {:?}", incoming_event);

        let IotaGossipHandlerInEvent { origin } = incoming_event;

        // We only send the upgrade request if this handler belongs to an outbound connection.
        if origin == Origin::Outbound {
            let send_request = ProtocolsHandlerEvent::OutboundSubstreamRequest {
                protocol: SubstreamProtocol::new(IotaGossipProtocolUpgrade::new(self.info.clone()), ()),
            };

            debug!("gossip handler: sending protocol upgrade request.");

            self.events.push_back(send_request);
        }
    }

    /// **libp2p docs**:
    ///
    /// Injects the output of a successful upgrade on a new inbound substream.
    fn inject_fully_negotiated_inbound(&mut self, new_inbound: NegotiatedSubstream, _: Self::InboundOpenInfo) {
        let negotiated_inbound = ProtocolsHandlerEvent::Custom(IotaGossipHandlerEvent::UpgradeCompleted {
            substream: Box::new(new_inbound),
        });

        debug!("gossip handler: fully negotiated inbound.");

        self.events.push_back(negotiated_inbound);
    }

    /// **libp2p docs**:
    ///
    /// Injects the output of a successful upgrade on a new outbound substream.
    ///
    /// The second argument is the information that was previously passed to
    /// [`ProtocolsHandlerEvent::OutboundSubstreamRequest`].
    fn inject_fully_negotiated_outbound(&mut self, new_outbound: NegotiatedSubstream, _: Self::OutboundOpenInfo) {
        let negotiated_outbound = ProtocolsHandlerEvent::Custom(IotaGossipHandlerEvent::UpgradeCompleted {
            substream: Box::new(new_outbound),
        });

        debug!("gossip handler: fully negotiated outbound.");

        self.events.push_back(negotiated_outbound);
    }

    /// **libp2p docs**:
    ///
    /// Notifies the handler of a change in the address of the remote.
    fn inject_address_change(&mut self, new_address: &Multiaddr) {
        debug!("gossip handler: new address: {}", new_address);
    }

    /// **libp2p docs**:
    ///
    /// Indicates to the handler that upgrading an outbound substream to the given protocol has failed.
    fn inject_dial_upgrade_error(
        &mut self,
        _: Self::OutboundOpenInfo,
        e: ProtocolsHandlerUpgrErr<<Self::OutboundProtocol as OutboundUpgrade<NegotiatedSubstream>>::Error>,
    ) {
        debug!("gossip handler: outbound upgrade error: {:?}", e);

        // TODO: finish event management in case of an error.
        // self.events.push_back(ProtocolsHandlerEvent::Close(e));
    }

    /// **libp2p docs**:
    ///
    /// Indicates to the handler that upgrading an inbound substream to the given protocol has failed.
    fn inject_listen_upgrade_error(
        &mut self,
        _: Self::InboundOpenInfo,
        e: ProtocolsHandlerUpgrErr<<Self::InboundProtocol as InboundUpgradeSend>::Error>,
    ) {
        debug!("gossip handler: inbound upgrade error: {:?}", e);

        // TODO: finish event management in case of an error.
        // let err = match e {
        //     ProtocolsHandlerUpgrErr::Timeout => io::Error::new(io::ErrorKind::TimedOut, "timeout"),
        //     ProtocolsHandlerUpgrErr::Timer => io::Error::new(io::ErrorKind::TimedOut, "timer"),
        //     ProtocolsHandlerUpgrErr::Upgrade(err) => err,
        // };

        // self.events.push_back(ProtocolsHandlerEvent::Close(err));
    }

    /// **libp2p docs**:
    ///
    /// Returns until when the connection should be kept alive.
    ///
    /// This method is called by the `Swarm` after each invocation of
    /// [`ProtocolsHandler::poll`] to determine if the connection and the associated
    /// `ProtocolsHandler`s should be kept alive as far as this handler is concerned
    /// and if so, for how long.
    ///
    /// Returning [`KeepAlive::No`] indicates that the connection should be
    /// closed and this handler destroyed immediately.
    ///
    /// Returning [`KeepAlive::Until`] indicates that the connection may be closed
    /// and this handler destroyed after the specified `Instant`.
    ///
    /// Returning [`KeepAlive::Yes`] indicates that the connection should
    /// be kept alive until the next call to this method.
    ///
    /// > **Note**: The connection is always closed and the handler destroyed
    /// > when [`ProtocolsHandler::poll`] returns an error. Furthermore, the
    /// > connection may be closed for reasons outside of the control
    /// > of the handler.
    fn connection_keep_alive(&self) -> KeepAlive {
        self.keep_alive
    }

    /// **libp2p docs**:
    ///
    /// Should behave like `Stream::poll()`.
    #[allow(clippy::type_complexity)]
    fn poll(
        &mut self,
        _: &mut Context<'_>,
    ) -> Poll<ProtocolsHandlerEvent<Self::OutboundProtocol, Self::OutboundOpenInfo, Self::OutEvent, Self::Error>> {
        if let Some(event) = self.events.pop_front() {
            Poll::Ready(event)
        } else {
            Poll::Pending
        }
    }
}
