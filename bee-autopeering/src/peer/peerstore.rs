// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Persistent storage of discovered peers.

use super::{
    peer_id::PeerId,
    peerlist::{ActivePeer, ActivePeersList, ReplacementList},
    Peer,
};

use sled::{Batch, Db};

use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

const ACTIVE_PEERS_TREE: &str = "active_peers";
const REPLACEMENTS_TREE: &str = "replacements";

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
    fn store_all_replacements(&self, peers: &ReplacementList);
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
    fn store_all_replacements(&self, peers: &ReplacementList) {
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

/// The config for the Sled peer store.
pub type SledPeerStoreConfig = sled::Config;

/// The (persistent) Sled peer store.
#[derive(Clone)]
pub struct SledPeerStore {
    db: Db,
}

impl PeerStore for SledPeerStore {
    type Config = SledPeerStoreConfig;

    fn new(config: Self::Config) -> Self {
        let db = config.open().expect("error opening peerstore");

        db.open_tree("active_peers").expect("error opening tree");
        db.open_tree("replacements").expect("error opening tree");

        Self { db }
    }

    fn store_active(&self, active_peer: ActivePeer) {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE).expect("error opening tree");
        let key = *active_peer.peer_id();

        tree.insert(key, active_peer).expect("insert error");
    }
    fn store_all_active(&self, active_peers: &ActivePeersList) {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE).expect("error opening tree");

        let mut batch = Batch::default();
        active_peers
            .read()
            .iter()
            .for_each(|p| batch.insert(*p.peer_id(), p.clone()));

        tree.apply_batch(batch).expect("error applying batch");
    }
    fn store_replacement(&self, peer: Peer) {
        let tree = self.db.open_tree(REPLACEMENTS_TREE).expect("error opening tree");
        let key = *peer.peer_id();

        tree.insert(key, peer).expect("error inserting peer");
    }
    fn store_all_replacements(&self, replacements: &ReplacementList) {
        let replacements_tree = self.db.open_tree(REPLACEMENTS_TREE).expect("error opening tree");

        let mut batch = Batch::default();
        replacements
            .read()
            .iter()
            .for_each(|p| batch.insert(*p.peer_id(), p.clone()));

        replacements_tree.apply_batch(batch).expect("error applying batch");
    }
    fn contains(&self, peer_id: &PeerId) -> bool {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE).expect("error opening tree");
        if tree.contains_key(peer_id).expect("db error") {
            true
        } else {
            let tree = self.db.open_tree(REPLACEMENTS_TREE).expect("error opening tree");
            tree.contains_key(peer_id).expect("db error")
        }
    }
    fn fetch_active(&self, peer_id: &PeerId) -> Option<ActivePeer> {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE).expect("error opening tree");

        tree.get(peer_id).expect("db error").map(ActivePeer::from)
    }
    fn fetch_all_active(&self) -> Vec<ActivePeer> {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE).expect("error opening tree");

        tree.iter()
            .filter_map(|p| p.ok())
            .map(|(_, ivec)| ActivePeer::from(ivec))
            .collect::<Vec<_>>()
    }
    fn fetch_replacement(&self, peer_id: &PeerId) -> Option<Peer> {
        let tree = self.db.open_tree(REPLACEMENTS_TREE).expect("error opening tree");

        tree.get(peer_id).expect("db error").map(Peer::from)
    }
    fn fetch_all_replacements(&self) -> Vec<Peer> {
        let tree = self.db.open_tree(REPLACEMENTS_TREE).expect("error opening tree");

        tree.iter()
            .filter_map(|p| p.ok())
            .map(|(_, ivec)| Peer::from(ivec))
            .collect::<Vec<_>>()
    }
    fn delete(&self, _: &PeerId) -> bool {
        unimplemented!("no need for single entry removal at the moment")
    }
    fn delete_all(&self) {
        self.db
            .open_tree(ACTIVE_PEERS_TREE)
            .expect("error opening tree")
            .clear()
            .expect("error clearing tree");

        self.db
            .open_tree(REPLACEMENTS_TREE)
            .expect("error opening tree")
            .clear()
            .expect("error clearing tree");
    }
}
