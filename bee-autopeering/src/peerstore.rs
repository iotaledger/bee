// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    delay::{DelayFactory, DelayedRepeat},
    identity::PeerId,
    packet::OutgoingPacket,
    peer::Peer,
    request::RequestManager,
    server::ServerTx,
    task::ShutdownRx,
    time::{self, Timestamp},
};

use sled::Db;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

// Used to send, e.g. (re-)verification and discovery requests in certain intervals.
#[async_trait::async_trait]
impl<S: PeerStore> DelayedRepeat<0> for S {
    type Context = (RequestManager, ServerTx);
    type Cancel = ShutdownRx;
}

pub trait PeerStore: Clone + Send + Sync {
    type Config;

    fn new(config: Self::Config) -> Self;

    fn last_verification_request(&self, peer_id: &PeerId) -> Option<Timestamp>;
    fn last_verification_response(&self, peer_id: &PeerId) -> Option<Timestamp>;
    fn peer(&self, peer_id: &PeerId) -> Option<Peer>;
    fn peers(&self) -> Vec<Peer>;

    fn update_last_verification_request(&self, peer_id: PeerId);
    fn update_last_verification_response(&self, peer_id: PeerId);

    fn insert_peer(&self, peer: Peer) -> bool;
    fn remove_peer(&self, peer_id: &PeerId) -> bool;
}

#[derive(Clone)]
pub struct InMemoryPeerStore {
    inner: Arc<RwLock<InMemoryPeerStoreInner>>,
}

#[derive(Default)]
struct InMemoryPeerStoreInner {
    last_verif_requests: HashMap<PeerId, Timestamp>,
    last_verif_responses: HashMap<PeerId, Timestamp>,
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
    fn last_verification_request(&self, peer_id: &PeerId) -> Option<Timestamp> {
        self.read_inner().last_verif_requests.get(peer_id).copied()
    }
    fn last_verification_response(&self, peer_id: &PeerId) -> Option<Timestamp> {
        self.read_inner().last_verif_responses.get(peer_id).copied()
    }
    fn peer(&self, peer_id: &PeerId) -> Option<Peer> {
        self.read_inner().peers.get(peer_id).map(|p| p.clone())
    }
    fn peers(&self) -> Vec<Peer> {
        self.read_inner().peers.values().cloned().collect::<Vec<Peer>>()
    }
    fn update_last_verification_request(&self, peer_id: PeerId) {
        let _ = self
            .write_inner()
            .last_verif_requests
            .insert(peer_id, time::unix_now_secs());
    }
    fn update_last_verification_response(&self, peer_id: PeerId) {
        let _ = self
            .write_inner()
            .last_verif_responses
            .insert(peer_id, time::unix_now_secs());
    }
    fn insert_peer(&self, peer: Peer) -> bool {
        let peer_id = peer.peer_id();
        if let Some(_) = self.write_inner().peers.insert(peer_id, peer) {
            false
        } else {
            // return `true` if the peer is new
            true
        }
    }
    fn remove_peer(&self, peer_id: &PeerId) -> bool {
        if let Some(_) = self.write_inner().peers.remove(peer_id) {
            true
        } else {
            // return `false` if the peer didn't need removing
            false
        }
    }
}

pub struct SledPeerStoreConfig {
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
pub struct SledPeerStore {
    db: Db,
}

impl PeerStore for SledPeerStore {
    type Config = SledPeerStoreConfig;

    fn new(config: Self::Config) -> Self {
        let db = sled::open(config.file_path).expect("error opening db");
        db.open_tree("last_verification_requests");
        db.open_tree("last_verification_responses");
        db.open_tree("peers");

        Self { db }
    }
    fn last_verification_request(&self, peer_id: &PeerId) -> Option<Timestamp> {
        todo!()
    }
    fn last_verification_response(&self, peer_id: &PeerId) -> Option<Timestamp> {
        todo!()
    }
    fn peer(&self, peer_id: &PeerId) -> Option<Peer> {
        todo!()
    }
    fn peers(&self) -> Vec<Peer> {
        todo!()
    }
    fn update_last_verification_request(&self, peer_id: PeerId) {
        todo!()
    }
    fn update_last_verification_response(&self, peer_id: PeerId) {
        todo!()
    }
    fn insert_peer(&self, peer: Peer) -> bool {
        todo!()
    }
    fn remove_peer(&self, peer_id: &PeerId) -> bool {
        todo!()
    }
}
