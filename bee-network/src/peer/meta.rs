// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::swarm::protocols::gossip::io::GossipSender;

#[derive(Clone, Debug, Default)]
pub struct PeerState(Option<GossipSender>);

impl PeerState {
    pub fn new_connected(gossip_sender: GossipSender) -> Self {
        Self(Some(gossip_sender))
    }

    pub fn new_disconnected() -> Self {
        Self(None)
    }

    pub fn is_disconnected(&self) -> bool {
        self.0.is_none()
    }

    pub fn is_connected(&self) -> bool {
        self.0.is_some()
    }

    pub fn set_connected(&mut self, gossip_sender: GossipSender) -> Option<GossipSender> {
        self.0.replace(gossip_sender)
    }

    pub fn set_disconnected(&mut self) -> Option<GossipSender> {
        self.0.take()
    }
}
