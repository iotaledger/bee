// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::peer::{
    lists::{ActivePeer, ActivePeersList, ReplacementPeersList},
    peer_id::PeerId,
    stores::PeerStore,
    Peer,
};

use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

/// A non-persistent/in-memory peer store.
#[derive(Clone, Default)]
pub struct InMemoryPeerStore {
    inner: Arc<RwLock<InMemoryPeerStoreInner>>,
}

#[derive(Default)]
struct InMemoryPeerStoreInner {
    active_peers: HashMap<PeerId, ActivePeer>,
    replacements: HashMap<PeerId, Peer>,
}

impl InMemoryPeerStore {
    fn read(&self) -> RwLockReadGuard<InMemoryPeerStoreInner> {
        self.inner.read().expect("error getting read access")
    }

    fn write(&self) -> RwLockWriteGuard<InMemoryPeerStoreInner> {
        self.inner.write().expect("error getting write access")
    }
}

impl PeerStore for InMemoryPeerStore {
    type Config = ();

    fn new(_: Self::Config) -> Self {
        Self {
            inner: Default::default(),
        }
    }

    fn store_active(&self, peer: ActivePeer) {
        let peer_id = peer.peer_id();

        let mut write = self.write();

        let _ = write.replacements.remove(peer_id);
        let _ = write.active_peers.insert(*peer_id, peer);
    }

    fn store_all_active(&self, peers: &ActivePeersList) {
        let read = peers.read();
        let mut write = self.write();

        for (peer_id, peer) in read.iter().map(|p| (p.peer_id(), p)) {
            let _ = write.active_peers.insert(*peer_id, peer.clone());
        }
    }

    fn store_replacement(&self, peer: Peer) {
        let peer_id = peer.peer_id();

        let _ = self.write().active_peers.remove(peer_id);
        let _ = self.write().replacements.insert(*peer_id, peer);
    }

    fn store_all_replacements(&self, peers: &ReplacementPeersList) {
        let read = peers.read();
        let mut write = self.write();

        for (peer_id, peer) in read.iter().map(|p| (p.peer_id(), p)) {
            let _ = write.replacements.insert(*peer_id, peer.clone());
        }
    }

    fn contains(&self, peer_id: &PeerId) -> bool {
        let read = self.read();
        read.active_peers.contains_key(peer_id) || read.replacements.contains_key(peer_id)
    }

    fn fetch_active(&self, peer_id: &PeerId) -> Option<ActivePeer> {
        self.read().active_peers.get(peer_id).cloned()
    }

    fn fetch_all_active(&self) -> Vec<ActivePeer> {
        self.read().active_peers.iter().map(|(_, p)| p).cloned().collect()
    }

    fn fetch_replacement(&self, peer_id: &PeerId) -> Option<Peer> {
        self.read().replacements.get(peer_id).cloned()
    }

    fn fetch_all_replacements(&self) -> Vec<Peer> {
        self.read().replacements.iter().map(|(_, p)| p).cloned().collect()
    }

    fn delete(&self, peer_id: &PeerId) -> bool {
        let mut write = self.write();
        write.active_peers.remove(peer_id).is_some() || write.replacements.remove(peer_id).is_some()
    }

    fn delete_all(&self) {
        let mut write = self.write();
        write.active_peers.clear();
        write.replacements.clear();
    }
}
