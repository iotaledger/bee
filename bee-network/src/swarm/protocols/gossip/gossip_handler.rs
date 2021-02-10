use super::GossipConfig;
use crate::host::connections::Origin;

use futures::{channel::mpsc, StreamExt};
use libp2p::{
    swarm::{
        KeepAlive, NegotiatedSubstream, ProtocolsHandler, ProtocolsHandlerEvent, ProtocolsHandlerUpgrErr,
        SubstreamProtocol,
    },
    OutboundUpgrade,
};

use std::{collections::VecDeque, io, task::Poll};

pub struct GossipHandler {
    config: GossipConfig,
    streams: VecDeque<NegotiatedSubstream>,
    origin: mpsc::Receiver<Origin>,
}

impl GossipHandler {
    pub fn new(origin: mpsc::Receiver<Origin>) -> Self {
        Self {
            config: GossipConfig::default(),
            streams: VecDeque::default(),
            origin,
        }
    }
}

impl ProtocolsHandler for GossipHandler {
    type InEvent = ();
    type OutEvent = NegotiatedSubstream;
    type Error = io::Error;
    type InboundProtocol = GossipConfig;
    type OutboundProtocol = GossipConfig;
    type InboundOpenInfo = ();
    type OutboundOpenInfo = ();

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
        SubstreamProtocol::new(self.config.clone(), ())
    }

    fn inject_fully_negotiated_inbound(&mut self, stream: NegotiatedSubstream, _: Self::InboundOpenInfo) {
        self.streams.push_back(stream);
    }

    fn inject_fully_negotiated_outbound(&mut self, stream: NegotiatedSubstream, _: Self::OutboundOpenInfo) {
        self.streams.push_back(stream);
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

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<ProtocolsHandlerEvent<Self::OutboundProtocol, Self::OutboundOpenInfo, Self::OutEvent, Self::Error>> {
        match self.origin.poll_next_unpin(cx) {
            Poll::Ready(origin) => {
                if let Some(origin) = origin {
                    if origin.is_outbound() {
                        let ev = ProtocolsHandlerEvent::OutboundSubstreamRequest {
                            protocol: SubstreamProtocol::new(self.config.clone(), ()),
                        };
                        Poll::Ready(ev)
                    } else {
                        if let Some(stream) = self.streams.pop_front() {
                            Poll::Ready(ProtocolsHandlerEvent::Custom(stream))
                        } else {
                            Poll::Pending
                        }
                    }
                } else {
                    Poll::Pending
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
