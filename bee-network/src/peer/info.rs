// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::Multiaddr;

/// Additional information about a peer.
#[derive(Clone, Debug)]
pub struct PeerInfo {
    /// The peer's address.
    pub address: Multiaddr,
    /// The peer's alias.
    pub alias: String,
    /// The type of relation we have with this peer.
    pub relation: PeerRelation,
}

/// Describes the relation with a peer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeerRelation {
    /// Represents a persistent peer. If the connection to such a peer drops, the network will try to reconnect.
    Known,
    /// Represents an ephemeral peer. If the connection to such a peer drops, the network won't try to reconnect.
    Unknown,
}

impl PeerRelation {
    /// Returns whether the peer is known.
    pub fn is_known(&self) -> bool {
        self.eq(&Self::Known)
    }

    /// Returns whether the peer is unknown.
    pub fn is_unknown(&self) -> bool {
        self.eq(&Self::Unknown)
    }

    /// Upgrades the peer relations.
    pub fn upgrade(&mut self) {
        if self.is_unknown() {
            *self = Self::Known;
        }
    }

    /// Downgrades the peer relation.
    pub fn downgrade(&mut self) {
        if self.is_known() {
            *self = Self::Unknown;
        }
    }
}

#[cfg(feature = "node")]
pub(super) mod peerstate {

    use crate::swarm::protocols::gossip::GossipSender;

    #[derive(Clone, Debug, Default)]
    pub struct PeerState(Option<GossipSender>);

    impl PeerState {
        pub fn connected(gossip_sender: GossipSender) -> Self {
            Self(Some(gossip_sender))
        }

        pub fn disconnected() -> Self {
            Self(None)
        }

        pub fn is_disconnected(&self) -> bool {
            self.0.is_none()
        }

        pub fn is_connected(&self) -> bool {
            self.0.is_some()
        }

        pub fn set_connected(&mut self, gossip_sender: GossipSender) {
            self.0.replace(gossip_sender);
        }

        pub fn set_disconnected(&mut self) -> Option<GossipSender> {
            self.0.take()
        }
    }
}
