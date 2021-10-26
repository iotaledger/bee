// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{identity::PeerId, ring::PeerRing};

use std::{
    collections::HashSet,
    fmt,
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

// Maximum number of peers that can be managed.
const DEFAULT_MAX_MANAGED: usize = 1000;
// Maximum number of peers kept in the replacement list.
const DEFAULT_MAX_REPLACEMENTS: usize = 10;

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

type ActivePeersListInner = PeerRing<ActivePeerEntry, DEFAULT_MAX_MANAGED>;

// Thread-safe active peer list.
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

// Non thread-safe replacement list.
pub(crate) type ReplacementList = PeerRing<PeerId, DEFAULT_MAX_REPLACEMENTS>;
// Non thread-safe master peer list.
pub(crate) type MasterPeersList = HashSet<PeerId>;

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

    pub fn incr_verified_count(&mut self) {
        self.verified_count += 1;
    }

    pub fn reset_verified_count(&mut self) {
        self.verified_count = 0;
    }

    pub fn last_new_peers(&self) -> usize {
        self.last_new_peers
    }

    pub fn incr_last_new_peers(&mut self) {
        self.last_new_peers += 1;
    }

    pub fn reset_last_new_peers(&mut self) {
        self.last_new_peers = 0;
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
