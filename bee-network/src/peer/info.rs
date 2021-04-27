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
    /// The type of relation we have with this peer.
    pub relation: PeerRelation,
}

/// Describes the relation with a peer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeerRelation {
    /// Represents a known peer.
    Known,
    /// Represents an unknown peer.
    Unknown,
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
}
