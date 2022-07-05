// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashMap,
    convert::Infallible,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::peer::{
    lists::{ActivePeer, ActivePeersList, ReplacementPeersList},
    peer_id::PeerId,
    stores::PeerStore,
    Peer,
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

    type Error = Infallible;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: Default::default(),
        })
    }

    fn store_active(&self, peer: ActivePeer) -> Result<(), Self::Error> {
        let peer_id = peer.peer_id();

        let mut write = self.write();

        let _ = write.replacements.remove(peer_id);
        let _ = write.active_peers.insert(*peer_id, peer);

        Ok(())
    }

    fn store_all_active(&self, peers: &ActivePeersList) -> Result<(), Self::Error> {
        let read = peers.read();
        let mut write = self.write();
        let peers = read.iter().map(|p| (p.peer_id(), p));

        for (peer_id, peer) in peers {
            let _ = write.active_peers.insert(*peer_id, peer.clone());
        }

        Ok(())
    }

    fn store_replacement(&self, peer: Peer) -> Result<(), Self::Error> {
        let peer_id = peer.peer_id();

        let _ = self.write().active_peers.remove(peer_id);
        let _ = self.write().replacements.insert(*peer_id, peer);

        Ok(())
    }

    fn store_all_replacements(&self, peers: &ReplacementPeersList) -> Result<(), Self::Error> {
        let read = peers.read();
        let mut write = self.write();
        let peers = read.iter().map(|p| (p.peer_id(), p));

        for (peer_id, peer) in peers {
            let _ = write.replacements.insert(*peer_id, peer.clone());
        }

        Ok(())
    }

    fn contains(&self, peer_id: &PeerId) -> Result<bool, Self::Error> {
        let read = self.read();
        Ok(read.active_peers.contains_key(peer_id) || read.replacements.contains_key(peer_id))
    }

    fn fetch_active(&self, peer_id: &PeerId) -> Result<Option<ActivePeer>, Self::Error> {
        Ok(self.read().active_peers.get(peer_id).cloned())
    }

    fn fetch_all_active(&self) -> Result<Vec<ActivePeer>, Self::Error> {
        Ok(self.read().active_peers.iter().map(|(_, p)| p).cloned().collect())
    }

    fn fetch_replacement(&self, peer_id: &PeerId) -> Result<Option<Peer>, Self::Error> {
        Ok(self.read().replacements.get(peer_id).cloned())
    }

    fn fetch_all_replacements(&self) -> Result<Vec<Peer>, Self::Error> {
        Ok(self.read().replacements.iter().map(|(_, p)| p).cloned().collect())
    }

    fn delete(&self, peer_id: &PeerId) -> Result<bool, Self::Error> {
        let mut write = self.write();
        Ok(write.active_peers.remove(peer_id).is_some() || write.replacements.remove(peer_id).is_some())
    }

    fn delete_all(&self) -> Result<(), Self::Error> {
        let mut write = self.write();
        write.active_peers.clear();
        write.replacements.clear();

        Ok(())
    }
}
