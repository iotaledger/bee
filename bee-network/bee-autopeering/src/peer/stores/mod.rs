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
pub use in_memory::*;
#[cfg(feature = "rocksdb")]
pub use self::rocksdb::*;
#[cfg(feature = "sled")]
pub use self::sled::*;

use super::{
    lists::{ActivePeer, ActivePeersList, ReplacementPeersList},
    peer_id::PeerId,
    Peer,
};

/// Mandatory functionality of any peer store.
pub trait PeerStore: Clone + Send + Sync {
    /// The peer store configuration.
    type Config;

    /// Creates a new peer store from config.
    fn new(config: Self::Config) -> Self;

    /// Stores an active peer.
    fn store_active(&self, peer: ActivePeer);

    /// Stores all current active peers.
    fn store_all_active(&self, peers: &ActivePeersList);

    /// Stores a replacement peer.
    fn store_replacement(&self, peer: Peer);

    /// Stores all current replacement peers.
    fn store_all_replacements(&self, peers: &ReplacementPeersList);

    /// Whether the store contains the given peer.
    fn contains(&self, peer_id: &PeerId) -> bool;

    /// Fetches an active peer from its peer identity.
    fn fetch_active(&self, peer_id: &PeerId) -> Option<ActivePeer>;

    /// Fetches all active peers.
    fn fetch_all_active(&self) -> Vec<ActivePeer>;

    /// Fetches a replacement peer from its peer identity.
    fn fetch_replacement(&self, peer_id: &PeerId) -> Option<Peer>;

    /// Fetches all replacement peers.
    fn fetch_all_replacements(&self) -> Vec<Peer>;

    /// Deletes a stored peer.
    fn delete(&self, peer_id: &PeerId) -> bool;

    /// Deletes all stored peers.
    fn delete_all(&self);
}
