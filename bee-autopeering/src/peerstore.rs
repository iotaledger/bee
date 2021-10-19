// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{identity::PeerId, peer::Peer, time::Timestamp};

use sled::Db;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub trait PeerStore: Clone + Send + Sync {
    type Config;

    fn new(config: Self::Config) -> Self;

    fn last_ping(&self, peer_id: &PeerId) -> Option<Timestamp>;
    fn last_pong(&self, peer_id: &PeerId) -> Option<Timestamp>;
    fn peer(&self, peer_id: &PeerId) -> Option<Peer>;

    fn update_last_ping(&mut self, peer_id: PeerId, timestamp: Timestamp);
    fn update_last_pong(&mut self, peer_id: PeerId, timestamp: Timestamp);

    fn insert_peer(&mut self, peer: Peer) -> bool;
    fn remove_peer(&mut self, peer_id: &PeerId) -> bool;
}

#[derive(Clone)]
pub struct InMemoryPeerStore {
    inner: Arc<RwLock<InMemoryPeerStoreInner>>,
}

#[derive(Default)]
struct InMemoryPeerStoreInner {
    last_pings: HashMap<PeerId, Timestamp>,
    last_pongs: HashMap<PeerId, Timestamp>,
    peers: HashMap<PeerId, Peer>,
}

impl InMemoryPeerStore {
    fn read_inner(&self) -> RwLockReadGuard<InMemoryPeerStoreInner> {
        self.inner.read().expect("error getting read access")
    }

    fn write_inner(&self) -> RwLockWriteGuard<InMemoryPeerStoreInner> {
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

    fn last_ping(&self, peer_id: &PeerId) -> Option<Timestamp> {
        self.read_inner().last_pings.get(peer_id).map(|v| *v)
    }

    fn last_pong(&self, peer_id: &PeerId) -> Option<Timestamp> {
        self.read_inner().last_pongs.get(peer_id).map(|v| *v)
    }

    fn peer(&self, peer_id: &PeerId) -> Option<Peer> {
        self.read_inner().peers.get(peer_id).map(|p| p.clone())
    }

    fn update_last_ping(&mut self, peer_id: PeerId, timestamp: Timestamp) {
        let _ = self.write_inner().last_pings.insert(peer_id, timestamp);
    }

    fn update_last_pong(&mut self, peer_id: PeerId, timestamp: Timestamp) {
        let _ = self.write_inner().last_pongs.insert(peer_id, timestamp);
    }

    fn insert_peer(&mut self, peer: Peer) -> bool {
        let peer_id = peer.peer_id();
        if let Some(_) = self.write_inner().peers.insert(peer_id, peer) {
            false
        } else {
            // return `true` if the peer is new
            true
        }
    }

    fn remove_peer(&mut self, peer_id: &PeerId) -> bool {
        if let Some(_) = self.write_inner().peers.remove(peer_id) {
            true
        } else {
            // return `false` if the peer didn't need removing
            false
        }
    }
}

pub(crate) struct SledPeerStoreConfig {
    // Path docs:
    // // Note: this example does work on Windows
    // let path = Path::new("./foo/bar.txt");
    pub(crate) file_path: PathBuf,
}

impl SledPeerStoreConfig {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: PathBuf::from(file_path),
        }
    }
}

impl Default for SledPeerStoreConfig {
    fn default() -> Self {
        Self {
            file_path: PathBuf::from("./autopeering"),
        }
    }
}

#[derive(Clone)]
pub(crate) struct SledPeerStore {
    db: Db,
}

impl PeerStore for SledPeerStore {
    type Config = SledPeerStoreConfig;

    fn new(config: Self::Config) -> Self {
        let db = sled::open(config.file_path).expect("error opening db");
        db.open_tree("last_pings");
        db.open_tree("last_pongs");
        db.open_tree("peers");

        Self { db }
    }

    fn last_ping(&self, peer_id: &PeerId) -> Option<Timestamp> {
        // self.db.get(key)
        todo!()
    }

    fn last_pong(&self, peer_id: &PeerId) -> Option<Timestamp> {
        todo!()
    }

    fn peer(&self, peer_id: &PeerId) -> Option<Peer> {
        todo!()
    }

    fn update_last_ping(&mut self, peer_id: PeerId, timestamp: Timestamp) {
        todo!()
    }

    fn update_last_pong(&mut self, peer_id: PeerId, timestamp: Timestamp) {
        todo!()
    }

    fn insert_peer(&mut self, peer: Peer) -> bool {
        todo!()
    }

    fn remove_peer(&mut self, peer_id: &PeerId) -> bool {
        todo!()
    }
}
