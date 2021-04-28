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
