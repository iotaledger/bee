// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    peer_id::PeerId,
    peerlist::{ActivePeer, ActivePeersList, PeerMetrics, ReplacementList},
    Peer,
};

use crate::{
    delay::DelayFactory,
    local::service_map::AUTOPEERING_SERVICE_NAME,
    packet::{MessageType, OutgoingPacket},
    request::RequestManager,
    server::ServerTx,
    task::ShutdownRx,
    time::{self, Timestamp},
};

use sled::{Batch, Db, IVec};

use std::{
    collections::HashMap,
    iter,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};

const ACTIVE_PEERS_TREE: &str = "active_peers";
const REPLACEMENTS_TREE: &str = "replacements";

pub trait PeerStore: Clone + Send + Sync {
    type Config;

    fn new(config: Self::Config) -> Self;
    fn store_active(&self, peer: ActivePeer);
    fn store_all_active(&self, peers: &ActivePeersList);
    fn store_replacement(&self, peer: Peer);
    fn store_all_replacements(&self, peers: &ReplacementList);
    fn contains(&self, peer_id: &PeerId) -> bool;
    fn fetch_active(&self, peer_id: &PeerId) -> Option<ActivePeer>;
    fn fetch_all_active(&self) -> Vec<ActivePeer>;
    fn fetch_replacement(&self, peer_id: &PeerId) -> Option<Peer>;
    fn fetch_all_replacements(&self) -> Vec<Peer>;
    fn delete(&self, peer_id: &PeerId) -> bool;
    fn delete_all(&self);
}

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

    fn new(config: Self::Config) -> Self {
        Self {
            inner: Default::default(),
        }
    }
    fn store_active(&self, peer: ActivePeer) {
        let peer_id = peer.peer_id();

        let mut write = self.write();

        let _ = write.replacements.remove(peer_id);
        let _ = write.active_peers.insert(peer_id.clone(), peer);
    }
    fn store_all_active(&self, peers: &ActivePeersList) {
        let read = peers.read();
        let mut write = self.write();

        for (peer_id, peer) in read.iter().map(|p| (p.peer_id(), p)) {
            let _ = write.active_peers.insert(peer_id.clone(), peer.clone());
        }
    }
    fn store_replacement(&self, peer: Peer) {
        let peer_id = peer.peer_id();

        let _ = self.write().active_peers.remove(peer_id);
        let _ = self.write().replacements.insert(peer_id.clone(), peer);
    }
    fn store_all_replacements(&self, peers: &ReplacementList) {
        let read = peers.read();
        let mut write = self.write();

        for (peer_id, peer) in read.iter().map(|p| (p.peer_id(), p)) {
            let _ = write.replacements.insert(peer_id.clone(), peer.clone());
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

pub struct SledPeerStoreConfig {
    inner: sled::Config,
}

impl SledPeerStoreConfig {
    pub fn new(file_path: &str) -> Self {
        Self {
            inner: sled::Config::new().path(PathBuf::from(file_path)),
        }
    }
}

impl Default for SledPeerStoreConfig {
    fn default() -> Self {
        Self::new("./peerstore")
    }
}

#[derive(Clone)]
pub struct SledPeerStore {
    db: Db,
}

impl PeerStore for SledPeerStore {
    type Config = sled::Config;

    fn new(config: Self::Config) -> Self {
        let db = config.open().expect("error opening peerstore");

        db.open_tree("active_peers");
        db.open_tree("replacements");

        Self { db }
    }

    fn store_active(&self, active_peer: ActivePeer) {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE).expect("error opening tree");
        let key = active_peer.peer_id().clone();

        tree.insert(key, active_peer).expect("insert error");
    }
    fn store_all_active(&self, active_peers: &ActivePeersList) {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE).expect("error opening tree");

        let mut batch = Batch::default();
        active_peers
            .read()
            .iter()
            .for_each(|p| batch.insert(p.peer_id().clone(), p.clone()));

        tree.apply_batch(batch).expect("error applying batch");
    }
    fn store_replacement(&self, peer: Peer) {
        let tree = self.db.open_tree(REPLACEMENTS_TREE).expect("error opening tree");
        let key = peer.peer_id().clone();

        tree.insert(key, peer);
    }
    fn store_all_replacements(&self, replacements: &ReplacementList) {
        let replacements_tree = self.db.open_tree(REPLACEMENTS_TREE).expect("error opening tree");

        let mut batch = Batch::default();
        replacements
            .read()
            .iter()
            .for_each(|p| batch.insert(p.peer_id().clone(), p.clone()));

        replacements_tree.apply_batch(batch).expect("error applying batch");
    }
    fn contains(&self, peer_id: &PeerId) -> bool {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE).expect("error opening tree");
        if tree.contains_key(peer_id).expect("db error") {
            true
        } else {
            let tree = self.db.open_tree(REPLACEMENTS_TREE).expect("error opening tree");
            if tree.contains_key(peer_id).expect("db error") {
                true
            } else {
                false
            }
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
    fn delete(&self, peer_id: &PeerId) -> bool {
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
