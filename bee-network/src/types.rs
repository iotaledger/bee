// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p_core::Multiaddr;

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

// TODO: use `matches!`
impl PeerRelation {
    /// Returns whether the peer is known.
    pub fn is_known(&self) -> bool {
        matches!(self, Self::Known)
    }

    /// Returns whether the peer is unknown.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    ///
    pub fn set_known(&mut self) {
        *self = Self::Known;
    }

    ///
    pub fn set_unknown(&mut self) {
        *self = Self::Unknown;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_and_set_api_works() {
        let mut pr = PeerRelation::Unknown;
        assert!(pr.is_unknown());

        pr.set_known();
        assert!(pr.is_known());

        pr.set_unknown();
        assert!(pr.is_unknown());
    }
}
