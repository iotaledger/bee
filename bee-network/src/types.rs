// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p_core::Multiaddr;

/// Additional information about a peer.
#[derive(Clone, Debug)]
pub struct PeerInfo {
    // TODO: rename to `multiaddr`
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
