// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::GossipUpgrade;
use crate::host::Origin;

use libp2p::{
    swarm::{
        KeepAlive, NegotiatedSubstream, ProtocolsHandler, ProtocolsHandlerEvent, ProtocolsHandlerUpgrErr,
        SubstreamProtocol,
    },
    OutboundUpgrade,
};
use log::trace;
use tokio::sync::mpsc::Receiver;

use std::{io, task::Poll};

pub struct GossipHandler {
    origin_rx: Receiver<Origin>,
    origin: Option<Origin>,
    stream: Option<NegotiatedSubstream>,
}

impl GossipHandler {
    pub fn new(origin_rx: Receiver<Origin>) -> Self {
        Self {
            origin_rx,
            origin: None,
            stream: None,
        }
    }
}

impl ProtocolsHandler for GossipHandler {
    type InEvent = ();
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
        trace!("HANDLER: negotiated inbound");
        self.stream.replace(stream);
    }

    fn inject_fully_negotiated_outbound(&mut self, stream: NegotiatedSubstream, _: Self::OutboundOpenInfo) {
        trace!("HANDLER: negotiated outbound");
        self.stream.replace(stream);
    }

    fn inject_event(&mut self, _event: Self::InEvent) {}

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
        cx: &mut std::task::Context<'_>,
    ) -> Poll<ProtocolsHandlerEvent<Self::OutboundProtocol, Self::OutboundOpenInfo, Self::OutEvent, Self::Error>> {
        if self.origin.is_none() {
            match self.origin_rx.poll_recv(cx) {
                Poll::Ready(Some(origin)) => {
                    self.origin.replace(origin);
                    if origin.is_outbound() {
                        let request_sent_event = ProtocolsHandlerEvent::OutboundSubstreamRequest {
                            protocol: SubstreamProtocol::new(GossipUpgrade::default(), ()),
                        };
                        trace!("HANDLER: request sent event");
                        return Poll::Ready(request_sent_event);
                    } else {
                        return Poll::Pending;
                    }
                }
                _ => return Poll::Pending,
            }
        }

        if let Some(stream) = self.stream.take() {
            trace!("HANDLER: stream result event");
            Poll::Ready(ProtocolsHandlerEvent::Custom(stream))
        } else {
            Poll::Pending
        }
    }
}
