// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::upgrade::GossipUpgrade;

use crate::network::meta::Origin;

use libp2p::{
    swarm::{
        KeepAlive, NegotiatedSubstream, ProtocolsHandler, ProtocolsHandlerEvent, ProtocolsHandlerUpgrErr,
        SubstreamProtocol,
    },
    OutboundUpgrade,
};
use log::trace;

use std::{io, task::Poll};

pub struct GossipHandler {
    origin: Origin,
    stream: Option<NegotiatedSubstream>,
    sent_request: bool,
}

impl GossipHandler {
    pub fn new(origin: Origin) -> Self {
        Self {
            origin,
            stream: None,
            sent_request: false,
        }
    }
}

impl ProtocolsHandler for GossipHandler {
    type InEvent = Origin;
    type OutEvent = NegotiatedSubstream;
    type Error = io::Error;
    type InboundProtocol = GossipUpgrade;
    type OutboundProtocol = GossipUpgrade;
    type InboundOpenInfo = ();
    type OutboundOpenInfo = ();

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
        SubstreamProtocol::new(GossipUpgrade::default(), ())
    }

    fn inject_fully_negotiated_inbound(&mut self, stream: NegotiatedSubstream, _: Self::InboundOpenInfo) {
        trace!("handler: negotiated inbound");
        self.stream.replace(stream);
    }

    fn inject_fully_negotiated_outbound(&mut self, stream: NegotiatedSubstream, _: Self::OutboundOpenInfo) {
        trace!("handler: negotiated outbound");
        self.stream.replace(stream);
    }

    fn inject_event(&mut self, event: Self::InEvent) {
        trace!("handler: in event: {}", event);
        self.origin = event;
    }

    fn inject_dial_upgrade_error(
        &mut self,
        _: Self::OutboundOpenInfo,
        _error: ProtocolsHandlerUpgrErr<<Self::OutboundProtocol as OutboundUpgrade<NegotiatedSubstream>>::Error>,
    ) {
    }

    fn connection_keep_alive(&self) -> libp2p::swarm::KeepAlive {
        KeepAlive::Yes
    }

    #[allow(clippy::type_complexity)]
    fn poll(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<ProtocolsHandlerEvent<Self::OutboundProtocol, Self::OutboundOpenInfo, Self::OutEvent, Self::Error>> {
        // NB: It is important to send the request to the peer only once and especially by convention only if the
        // connection is outbound.

        if !self.sent_request && self.origin.is_outbound() {
            self.sent_request = true;

            let request_sent_event = ProtocolsHandlerEvent::OutboundSubstreamRequest {
                protocol: SubstreamProtocol::new(GossipUpgrade::default(), ()),
            };
            trace!("handler: request sent event");
            return Poll::Ready(request_sent_event);
        }

        if let Some(stream) = self.stream.take() {
            trace!("handler: stream result event");
            Poll::Ready(ProtocolsHandlerEvent::Custom(stream))
        } else {
            Poll::Pending
        }
    }
}
