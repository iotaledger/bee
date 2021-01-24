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
use tokio_stream::wrappers::UnboundedReceiverStream;

/// A shorthand for an unbounded channel sender.
pub type MessageSender = mpsc::UnboundedSender<Vec<u8>>;

/// A shorthand for an unbounded channel receiver.
pub type MessageReceiver = UnboundedReceiverStream<Vec<u8>>;

pub fn channel() -> (MessageSender, MessageReceiver) {
    let (sender, receiver) = mpsc::unbounded_channel();
    (sender, UnboundedReceiverStream::new(receiver))
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
