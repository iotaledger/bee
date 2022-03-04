// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p_core::Multiaddr;

/// Additional information about a peer.
#[derive(Clone, Debug)]
pub struct PeerInfo {
    /// The peer's address.
    pub address: Multiaddr,
    /// The peer's alias.
    pub alias: String,
    /// The type of relation the node has with this peer.
    pub relation: PeerRelation,
}

/// Describes the type of a peer.
#[derive(Debug)]
pub enum PeerType {
    /// Represents a manually added peer.
    Manual,
    /// Represents an automatically added peer.
    Auto,
}

/// Describes the relation with a peer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeerRelation {
    /// Represents a known peer.
    Known,
    /// Represents an unknown peer.
    Unknown,
    /// Represents a discovered peer.
    Discovered,
}

impl From<PeerType> for PeerRelation {
    fn from(ty: PeerType) -> Self {
        match ty {
            PeerType::Manual => Self::Known,
            PeerType::Auto => Self::Discovered,
        }
    }
}

impl PeerRelation {
    /// Returns whether the peer is known.
    pub fn is_known(&self) -> bool {
        matches!(self, Self::Known)
    }

    /// Returns whether the peer is unknown.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    /// Returns whether the peer is discovered.
    pub fn is_discovered(&self) -> bool {
        matches!(self, Self::Discovered)
    }

    /// Sets the relation to "known".
    pub fn set_known(&mut self) {
        *self = Self::Known;
    }

    /// Sets the relation to "unknown".
    pub fn set_unknown(&mut self) {
        *self = Self::Unknown;
    }

    /// Sets the relation to "discovered".
    pub fn set_discovered(&mut self) {
        *self = Self::Discovered;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_and_set_api() {
        let mut pr = PeerRelation::Unknown;
        assert!(pr.is_unknown());

        pr.set_known();
        assert!(pr.is_known());

        pr.set_unknown();
        assert!(pr.is_unknown());

        pr.set_discovered();
        assert!(pr.is_discovered())
    }
}
