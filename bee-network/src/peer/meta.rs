// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::swarm::protocols::gossip::io::GossipSender;

use std::mem::take;

#[derive(Clone, Debug)]
pub enum PeerState {
    Disconnected,
    Connected(GossipSender),
}

impl Default for PeerState {
    fn default() -> Self {
        Self::Disconnected
    }
}

impl PeerState {
    pub fn is_disconnected(&self) -> bool {
        matches!(self, Self::Disconnected)
    }

    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected(_))
    }

    pub fn to_connected(&mut self, gossip_sender: GossipSender) -> Option<GossipSender> {
        *self = Self::Connected(gossip_sender);
        None
    }

    pub fn to_disconnected(&mut self) -> Option<GossipSender> {
        match take(self) {
            Self::Disconnected => None,
            Self::Connected(sender) => Some(sender),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swarm::protocols::gossip::io::gossip_channel;

    #[test]
    fn new_peer_state() {
        let peerstate = PeerState::default();

        assert!(peerstate.is_disconnected());
    }

    #[test]
    fn peer_state_change() {
        let mut peerstate = PeerState::Disconnected;
        let (tx, _rx) = gossip_channel();

        peerstate.to_connected(tx);
        assert!(peerstate.is_connected());

        assert!(peerstate.to_disconnected().is_some());
        assert!(peerstate.is_disconnected());
        assert!(peerstate.to_disconnected().is_none());
    }
}
