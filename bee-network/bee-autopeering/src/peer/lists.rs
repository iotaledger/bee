// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{peer_id::PeerId, Peer};

use crate::{
    discovery::manager::VERIFICATION_EXPIRATION,
    time::{self, Timestamp},
};

use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};

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
type ReplacementPeersListInner = PeerRing<Peer, DEFAULT_MAX_REPLACEMENTS>;
type EntryPeersListInner = HashSet<PeerId>;

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

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("serialization error")
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).expect("deserialization error")
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
        bincode::serialize(&peer).expect("serialization error").into()
    }
}

impl From<sled::IVec> for ActivePeer {
    fn from(bytes: sled::IVec) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("deserialization error")
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

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let peer = seq
            .next_element::<Peer>()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

        let metrics = seq
            .next_element::<PeerMetrics>()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

        Ok(ActivePeer { peer, metrics })
    }
}

// TODO: stop exposing lock guards.
#[derive(Clone, Default)]
pub struct ActivePeersList {
    inner: Arc<RwLock<ActivePeersListInner>>,
}

impl ActivePeersList {
    pub(crate) fn read(&self) -> RwLockReadGuard<ActivePeersListInner> {
        // Panice: we don't allow poisened locks.
        self.inner.read().expect("error getting read access")
    }

    pub(crate) fn write(&self) -> RwLockWriteGuard<ActivePeersListInner> {
        // Panice: we don't allow poisened locks.
        self.inner.write().expect("error getting write access")
    }
}

#[derive(Clone, Default)]
pub struct ReplacementPeersList {
    inner: Arc<RwLock<ReplacementPeersListInner>>,
}

impl ReplacementPeersList {
    pub(crate) fn read(&self) -> RwLockReadGuard<ReplacementPeersListInner> {
        self.inner.read().expect("error getting read access")
    }

    pub(crate) fn write(&self) -> RwLockWriteGuard<ReplacementPeersListInner> {
        self.inner.write().expect("error getting write access")
    }
}

#[derive(Clone, Default)]
pub(crate) struct EntryPeersList {
    inner: Arc<RwLock<EntryPeersListInner>>,
}

impl EntryPeersList {
    pub(crate) fn read(&self) -> RwLockReadGuard<EntryPeersListInner> {
        self.inner.read().expect("error getting read access")
    }

    pub(crate) fn write(&self) -> RwLockWriteGuard<EntryPeersListInner> {
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
    pub(crate) fn verified_count(&self) -> usize {
        self.verified_count
    }

    /// Increments the verified counter, and returns the new value.
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

    pub(crate) fn set_last_verif_request_timestamp(&mut self) {
        self.last_verif_request = time::unix_now_secs();
    }

    pub(crate) fn set_last_verif_response_timestamp(&mut self) {
        self.last_verif_response = time::unix_now_secs();
    }

    pub(crate) fn is_verified(&self) -> bool {
        time::since(self.last_verif_response).expect("system clock error") < VERIFICATION_EXPIRATION.as_secs()
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

    pub(crate) fn get_newest_mut(&mut self) -> Option<&mut P> {
        self.0.get_mut(0)
    }

    // NOTE: need to be atomic operations
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

    pub(crate) fn iter(&self) -> impl Iterator<Item = &P> {
        self.0.iter()
    }

    #[cfg(test)]
    pub(crate) fn rotate_forwards(&mut self) {
        self.0.rotate_right(1);
    }
}

impl<P, const N: usize> Default for PeerRing<P, N> {
    fn default() -> Self {
        Self(VecDeque::with_capacity(N))
    }
}
