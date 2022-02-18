// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Persistent storage of discovered peers.

#[cfg(feature = "in-memory")]
mod in_memory;
#[cfg(feature = "rocksdb")]
mod rocksdb;
#[cfg(feature = "sled")]
mod sled;

#[cfg(feature = "in-memory")]
pub use self::in_memory::*;
#[cfg(feature = "rocksdb")]
pub use self::rocksdb::*;
#[cfg(feature = "sled")]
pub use self::sled::*;

use super::{
    lists::{ActivePeer, ActivePeersList, ReplacementPeersList},
    peer_id::PeerId,
    Peer,
};

use std::error::Error;

/// Mandatory functionality of any peer store.
pub trait PeerStore: Clone + Send + Sync {
    /// The peer store configuration.
    type Config;

    /// Error raised when a peer store operation fails.
    type Error: Error + Send;

    /// Creates a new peer store from config.
    fn new(config: Self::Config) -> Result<Self, Self::Error>;

    /// Stores an active peer.
    fn store_active(&self, peer: ActivePeer) -> Result<(), Self::Error>;

    /// Stores all current active peers.
    fn store_all_active(&self, peers: &ActivePeersList) -> Result<(), Self::Error>;

    /// Stores a replacement peer.
    fn store_replacement(&self, peer: Peer) -> Result<(), Self::Error>;

    /// Stores all current replacement peers.
    fn store_all_replacements(&self, peers: &ReplacementPeersList) -> Result<(), Self::Error>;

    /// Whether the store contains the given peer.
    fn contains(&self, peer_id: &PeerId) -> Result<bool, Self::Error>;

    /// Fetches an active peer from its peer -> Result<(), Self::Error> identity.
    fn fetch_active(&self, peer_id: &PeerId) -> Result<Option<ActivePeer>, Self::Error>;

    /// Fetches all active peers.
    fn fetch_all_active(&self) -> Result<Vec<ActivePeer>, Self::Error>;

    /// Fetches a replacement peer from its peer identity.
    fn fetch_replacement(&self, peer_id: &PeerId) -> Result<Option<Peer>, Self::Error>;

    /// Fetches all replacement peers.
    fn fetch_all_replacements(&self) -> Result<Vec<Peer>, Self::Error>;

    /// Deletes a stored peer.
    fn delete(&self, peer_id: &PeerId) -> Result<bool, Self::Error>;

    /// Deletes all stored peers.
    fn delete_all(&self) -> Result<(), Self::Error>;
}
