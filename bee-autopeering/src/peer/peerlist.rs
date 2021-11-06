// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{peer_id::PeerId, Peer};

use crate::{
    discovery::manager::VERIFICATION_EXPIRATION_SECS,
    time::{self, Timestamp},
};

use serde::{de::Visitor, ser::SerializeStruct, Deserialize, Serialize};

use std::{
    collections::{HashSet, VecDeque},
    fmt,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

// Maximum number of peers that can be managed.
const DEFAULT_MAX_MANAGED: usize = 1000;
// Maximum number of peers kept in the replacement list.
const DEFAULT_MAX_REPLACEMENTS: usize = 10;

type ActivePeersListInner = PeerRing<ActivePeer, DEFAULT_MAX_MANAGED>;
type ReplacementListInner = PeerRing<Peer, DEFAULT_MAX_REPLACEMENTS>;
type MasterPeersListInner = HashSet<PeerId>;

#[derive(Clone)]
pub struct ActivePeer {
    peer: Peer,
    metrics: PeerMetrics,
}

impl ActivePeer {
    pub(crate) fn new(peer: Peer) -> Self {
        Self {
            peer,
            metrics: PeerMetrics::default(),
        }
    }

    pub(crate) fn peer(&self) -> &Peer {
        &self.peer
    }

    pub(crate) fn peer_mut(&mut self) -> &mut Peer {
        &mut self.peer
    }

    pub(crate) fn peer_id(&self) -> &PeerId {
        self.peer.peer_id()
    }

    pub(crate) fn metrics(&self) -> &PeerMetrics {
        &self.metrics
    }

    pub(crate) fn metrics_mut(&mut self) -> &mut PeerMetrics {
        &mut self.metrics
    }

    pub(crate) fn into_peer(self) -> Peer {
        self.peer
    }
}

impl Eq for ActivePeer {}
impl PartialEq for ActivePeer {
    fn eq(&self, other: &Self) -> bool {
        self.peer.peer_id() == other.peer.peer_id()
    }
}

impl From<Peer> for ActivePeer {
    fn from(peer: Peer) -> Self {
        Self::new(peer)
    }
}

impl AsRef<PeerId> for ActivePeer {
    fn as_ref(&self) -> &PeerId {
        self.peer.peer_id()
    }
}

impl AsRef<Peer> for ActivePeer {
    fn as_ref(&self) -> &Peer {
        &self.peer
    }
}

impl From<ActivePeer> for sled::IVec {
    fn from(peer: ActivePeer) -> Self {
        let bytes = bincode::serialize(&peer).expect("serialization error");
        sled::IVec::from_iter(bytes.into_iter())
    }
}

impl From<sled::IVec> for ActivePeer {
    fn from(bytes: sled::IVec) -> Self {
        bincode::deserialize(&bytes).expect("deserialization error")
    }
}

impl<'de> Deserialize<'de> for ActivePeer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct("ActivePeer", &["peer", "metrics"], ActivePeerVisitor {})
    }
}

impl Serialize for ActivePeer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut this = serializer.serialize_struct("ActivePeer", 2)?;
        this.serialize_field("peer", &self.peer)?;
        this.serialize_field("metrics", &self.metrics)?;
        this.end()
    }
}

struct ActivePeerVisitor {}

impl<'de> Visitor<'de> for ActivePeerVisitor {
    type Value = ActivePeer;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("'ActivePeer'")
    }
}

#[derive(Clone, Default)]
pub struct ActivePeersList {
    inner: Arc<RwLock<ActivePeersListInner>>,
}

impl ActivePeersList {
    pub(crate) fn read(&self) -> RwLockReadGuard<ActivePeersListInner> {
        self.inner.read().expect("error getting read access")
    }

    pub(crate) fn write(&self) -> RwLockWriteGuard<ActivePeersListInner> {
        self.inner.write().expect("error getting write access")
    }
}

#[derive(Clone, Default)]
pub struct ReplacementList {
    inner: Arc<RwLock<ReplacementListInner>>,
}

impl ReplacementList {
    pub(crate) fn read(&self) -> RwLockReadGuard<ReplacementListInner> {
        self.inner.read().expect("error getting read access")
    }

    pub(crate) fn write(&self) -> RwLockWriteGuard<ReplacementListInner> {
        self.inner.write().expect("error getting write access")
    }
}

#[derive(Clone, Default)]
pub(crate) struct MasterPeersList {
    inner: Arc<RwLock<MasterPeersListInner>>,
}

impl MasterPeersList {
    // TODO: revisit dead code
    #[allow(dead_code)]
    pub(crate) fn new(peers: MasterPeersListInner) -> Self {
        Self {
            inner: Arc::new(RwLock::new(peers)),
        }
    }

    pub(crate) fn read(&self) -> RwLockReadGuard<MasterPeersListInner> {
        self.inner.read().expect("error getting read access")
    }

    pub(crate) fn write(&self) -> RwLockWriteGuard<MasterPeersListInner> {
        self.inner.write().expect("error getting write access")
    }
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
pub(crate) struct PeerMetrics {
    // how often that peer has been re-verified
    verified_count: usize,
    // number of returned new peers when queried the last time
    last_new_peers: usize,
    // timestamp of last verification request received
    last_verif_request: Timestamp,
    // timestamp of last verification request received
    last_verif_response: Timestamp,
}

impl PeerMetrics {
    // TODO: revisit dead code
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self {
            verified_count: 0,
            last_new_peers: 0,
            last_verif_request: 0,
            last_verif_response: 0,
        }
    }

    pub(crate) fn verified_count(&self) -> usize {
        self.verified_count
    }

    /// Inrements the verified counter, and returns the new value.
    pub(crate) fn increment_verified_count(&mut self) -> usize {
        self.verified_count += 1;
        self.verified_count
    }

    pub(crate) fn reset_verified_count(&mut self) {
        self.verified_count = 0;
    }

    pub(crate) fn last_new_peers(&self) -> usize {
        self.last_new_peers
    }

    pub(crate) fn set_last_new_peers(&mut self, last_new_peers: usize) {
        self.last_new_peers = last_new_peers;
    }

    // TODO: revisit dead code
    #[allow(dead_code)]
    pub(crate) fn last_verif_request_timestamp(&self) -> Timestamp {
        self.last_verif_request
    }

    pub(crate) fn set_last_verif_request_timestamp(&mut self) {
        self.last_verif_request = time::unix_now_secs();
    }

    // TODO: revisit dead code
    #[allow(dead_code)]
    pub(crate) fn last_verif_response_timestamp(&self) -> Timestamp {
        self.last_verif_response
    }

    pub(crate) fn set_last_verif_response_timestamp(&mut self) {
        self.last_verif_response = time::unix_now_secs();
    }

    pub(crate) fn is_verified(&self) -> bool {
        time::since(self.last_verif_response).expect("system clock error") < VERIFICATION_EXPIRATION_SECS
    }

    // TODO: revisit dead code
    #[allow(dead_code)]
    pub(crate) fn has_verified(&self) -> bool {
        time::since(self.last_verif_request).expect("system clock error") < VERIFICATION_EXPIRATION_SECS
    }
}

impl fmt::Debug for PeerMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PeerMetrics")
            .field("verified_count", &self.verified_count)
            .field("last_new_peers", &self.last_new_peers)
            .field("last_verif_request", &self.last_verif_request)
            .field("last_verif_response", &self.last_verif_response)
            .finish()
    }
}

/// TODO: consider using `IndexMap` for faster search.
#[derive(Clone)]
pub(crate) struct PeerRing<P, const N: usize>(VecDeque<P>);

impl<P: AsRef<PeerId>, const N: usize> PeerRing<P, N> {
    // TODO: revisit dead code
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Returns `false`, if the list already contains the id, otherwise `true`.
    ///
    /// The newest item will be at index `0`, the oldest at index `n`.
    pub(crate) fn insert(&mut self, item: P) -> bool {
        if self.contains(item.as_ref()) {
            false
        } else {
            if self.is_full() {
                self.remove_oldest();
            }
            self.0.push_front(item);
            true
        }
    }

    pub(crate) fn remove_oldest(&mut self) -> Option<P> {
        self.0.pop_back()
    }

    pub(crate) fn remove(&mut self, peer_id: &PeerId) -> Option<P> {
        if let Some(index) = self.find_index(peer_id) {
            self.remove_at(index)
        } else {
            None
        }
    }

    pub(crate) fn remove_at(&mut self, index: usize) -> Option<P> {
        self.0.remove(index)
    }

    pub(crate) fn contains(&self, peer_id: &PeerId) -> bool {
        self.0.iter().any(|v| v.as_ref() == peer_id)
    }

    pub(crate) fn find_index(&self, peer_id: &PeerId) -> Option<usize> {
        self.0.iter().position(|v| v.as_ref() == peer_id)
    }

    pub(crate) fn find(&self, peer_id: &PeerId) -> Option<&P> {
        self.find_index(peer_id).map(|index| self.get(index)).flatten()
    }

    pub(crate) fn find_mut(&mut self, peer_id: &PeerId) -> Option<&mut P> {
        let index = self.find_index(peer_id);
        if let Some(index) = index {
            self.get_mut(index)
        } else {
            None
        }
    }

    pub(crate) fn get(&self, index: usize) -> Option<&P> {
        self.0.get(index)
    }

    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut P> {
        self.0.get_mut(index)
    }

    // TODO: revisit dead code
    #[allow(dead_code)]
    pub(crate) fn get_newest(&self) -> Option<&P> {
        self.0.get(0)
    }

    pub(crate) fn get_newest_mut(&mut self) -> Option<&mut P> {
        self.0.get_mut(0)
    }

    // TODO: revisit dead code
    /// Moves `peer_id` to the front of the list.
    ///
    /// Returns `false` if the `peer_id` is not found in the list, and thus, cannot be made the newest.
    #[allow(dead_code)]
    pub(crate) fn set_newest(&mut self, peer_id: &PeerId) -> bool {
        if let Some(mid) = self.find_index(peer_id) {
            if mid > 0 {
                self.0.rotate_left(mid);
            }
            true
        } else {
            false
        }
    }

    // needs to be atomic
    // TODO: revisit dead code
    #[allow(dead_code)]
    pub(crate) fn set_newest_and_get(&mut self, peer_id: &PeerId) -> Option<&P> {
        if let Some(mid) = self.find_index(peer_id) {
            if mid > 0 {
                self.0.rotate_left(mid);
            }
            self.get_newest()
        } else {
            None
        }
    }

    // needs to be atomic
    pub(crate) fn set_newest_and_get_mut(&mut self, peer_id: &PeerId) -> Option<&mut P> {
        if let Some(mid) = self.find_index(peer_id) {
            if mid > 0 {
                self.0.rotate_left(mid);
            }
            self.get_newest_mut()
        } else {
            None
        }
    }

    pub(crate) fn get_oldest(&self) -> Option<&P> {
        if self.0.is_empty() {
            None
        } else {
            self.0.get(self.0.len() - 1)
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn is_full(&self) -> bool {
        self.len() >= N
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // TODO: revisit dead code
    // TODO: mark as 'const fn' once stable.
    // Compiler error hints to issue #57563 <https://github.com/rust-lang/rust/issues/57563>.
    #[allow(dead_code)]
    pub(crate) fn max_size(&self) -> usize {
        N
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &P> {
        self.0.iter()
    }
}

impl<P, const N: usize> Default for PeerRing<P, N> {
    fn default() -> Self {
        Self(VecDeque::with_capacity(N))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<P: AsRef<PeerId>, const N: usize> PeerRing<P, N> {
        // TODO: revisit dead code
        #[allow(dead_code)]
        pub(crate) fn rotate_backwards(&mut self) {
            self.0.rotate_left(1);
        }

        pub(crate) fn rotate_forwards(&mut self) {
            self.0.rotate_right(1);
        }
    }
}
