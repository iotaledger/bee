// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod banned;
mod errors;
mod list;
mod manager;

pub use banned::*;
pub use errors::Error;
pub use list::*;
pub use manager::*;

use tokio::sync::mpsc;

pub type DataSender = mpsc::UnboundedSender<Vec<u8>>;
pub type DataReceiver = mpsc::UnboundedReceiver<Vec<u8>>;

pub fn channel() -> (DataSender, DataReceiver) {
    mpsc::unbounded_channel()
}

/// Describes the relation with a peer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PeerRelation {
    /// Represents a known peer.
    Known,
    /// Represents an unknown peer.
    Unknown,
    /// Represents a discovered peer.
    Discovered,
}

impl PeerRelation {
    /// Returns whether the peer is known.
    pub fn is_known(&self) -> bool {
        *self == PeerRelation::Known
    }

    /// Returns whether the peer is unknown.
    pub fn is_unknown(&self) -> bool {
        *self == PeerRelation::Unknown
    }

    /// Returns whether the peer is discovered.
    pub fn is_discovered(&self) -> bool {
        *self == PeerRelation::Discovered
    }
}
