// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{peer_id::PeerId, peerstore::PeerStore};
use crate::{
    command::{Command, CommandTx},
    discovery,
    request::RequestManager,
    server::ServerTx,
    task::{Repeat, Runnable, ShutdownRx},
};

use std::{
    collections::{vec_deque, HashSet, VecDeque},
    fmt,
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

// Maximum number of peers that can be managed.
const DEFAULT_MAX_MANAGED: usize = 1000;
// Maximum number of peers kept in the replacement list.
const DEFAULT_MAX_REPLACEMENTS: usize = 10;

type ActivePeersListInner = PeerRing<ActivePeerEntry, DEFAULT_MAX_MANAGED>;
type ReplacementListInner = PeerRing<PeerId, DEFAULT_MAX_REPLACEMENTS>;
type MasterPeersListInner = HashSet<PeerId>;

#[derive(Clone)]
pub(crate) struct ActivePeerEntry {
    peer_id: PeerId,
    metrics: PeerMetrics,
}

impl ActivePeerEntry {
    pub(crate) fn new(peer_id: PeerId) -> Self {
        Self {
            peer_id,
            metrics: PeerMetrics::default(),
        }
    }

    pub(crate) fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    pub(crate) fn metrics(&self) -> &PeerMetrics {
        &self.metrics
    }

    pub(crate) fn metrics_mut(&mut self) -> &mut PeerMetrics {
        &mut self.metrics
    }

    pub(crate) fn into_id(self) -> PeerId {
        self.peer_id
    }
}

impl Eq for ActivePeerEntry {}
impl PartialEq for ActivePeerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.peer_id == other.peer_id
    }
}

impl From<PeerId> for ActivePeerEntry {
    fn from(peer_id: PeerId) -> Self {
        Self::new(peer_id)
    }
}

#[derive(Clone, Default)]
pub(crate) struct ActivePeersList {
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
pub(crate) struct ReplacementList {
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

impl AsRef<PeerId> for PeerId {
    fn as_ref(&self) -> &PeerId {
        self
    }
}

impl AsRef<PeerId> for ActivePeerEntry {
    fn as_ref(&self) -> &PeerId {
        &self.peer_id
    }
}

#[derive(Clone, Copy, Default)]
pub struct PeerMetrics {
    // how often that peer has been re-verified
    verified_count: usize,
    // number of returned new peers when queried the last time
    last_new_peers: usize,
}

impl PeerMetrics {
    pub fn new(peer_id: PeerId) -> Self {
        Self {
            verified_count: 0,
            last_new_peers: 0,
        }
    }

    pub fn verified_count(&self) -> usize {
        self.verified_count
    }

    pub fn increment_verified_count(&mut self) -> usize {
        self.verified_count += 1;
        self.verified_count
    }

    pub fn reset_verified_count(&mut self) {
        self.verified_count = 0;
    }

    pub fn last_new_peers(&self) -> usize {
        self.last_new_peers
    }

    pub fn set_last_new_peers(&mut self, last_new_peers: usize) {
        self.last_new_peers = last_new_peers;
    }
}

impl fmt::Debug for PeerMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PeerMetrics")
            .field("verified_count", &self.verified_count)
            .field("last_new_peers", &self.last_new_peers)
            .finish()
    }
}

#[derive(Clone)]
pub(crate) struct PeerRing<P, const N: usize>(VecDeque<P>);

impl<P: AsRef<PeerId>, const N: usize> PeerRing<P, N> {
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

    pub(crate) fn get_newest(&self) -> Option<&P> {
        self.0.get(0)
    }

    pub(crate) fn get_newest_mut(&mut self) -> Option<&mut P> {
        self.0.get_mut(0)
    }

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
        self.0.get(self.0.len() - 1)
    }

    pub(crate) fn rotate_backwards(&mut self) {
        self.0.rotate_left(1);
    }

    pub(crate) fn rotate_forwards(&mut self) {
        self.0.rotate_right(1);
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

    // TODO: mark as 'const fn' once stable.
    // Compiler error hints to issue #57563 <https://github.com/rust-lang/rust/issues/57563>.
    pub(crate) fn max_size(&self) -> usize {
        N
    }

    pub(crate) fn iter(&self) -> vec_deque::Iter<P> {
        self.0.iter()
    }
}

impl<P, const N: usize> Default for PeerRing<P, N> {
    fn default() -> Self {
        Self(VecDeque::with_capacity(N))
    }
}
