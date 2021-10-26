// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    delay::{Command, Cronjob, DelayFactory},
    identity::PeerId,
    packet::{MessageType, OutgoingPacket},
    peer::{self, Peer},
    peerlist::PeerMetrics,
    request::RequestManager,
    server::ServerTx,
    service_map::AUTOPEERING_SERVICE_NAME,
    task::ShutdownRx,
    time::{self, Timestamp},
};

use sled::Db;

use std::{
    collections::HashMap,
    iter,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};

pub trait PeerStore: Clone + Send + Sync {
    type Config;

    fn new(config: Self::Config) -> Self;

    fn insert_peer(&self, peer: Peer) -> bool;
    fn remove_peer(&self, peer_id: &PeerId) -> bool;

    fn get_peer(&self, peer_id: &PeerId) -> Option<Peer>;
    fn get_peers(&self) -> Vec<Peer>;

    fn last_verification_request(&self, peer_id: &PeerId) -> Option<Timestamp>;
    fn last_verification_response(&self, peer_id: &PeerId) -> Option<Timestamp>;

    fn update_last_verification_request(&self, peer_id: PeerId);
    fn update_last_verification_response(&self, peer_id: PeerId);
    fn update_peer_metrics(&self, peer_id: &PeerId, metrics: impl Fn(&mut PeerMetrics));
}

#[derive(Clone, Default)]
pub struct InMemoryPeerStore {
    inner: Arc<RwLock<InMemoryPeerStoreInner>>,
}

#[derive(Default)]
struct InMemoryPeerStoreInner {
    last_verif_requests: HashMap<PeerId, Timestamp>,
    last_verif_responses: HashMap<PeerId, Timestamp>,
    peers: HashMap<PeerId, Peer>,
    metrics: HashMap<PeerId, PeerMetrics>,
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

    fn insert_peer(&self, peer: Peer) -> bool {
        let peer_id = peer.peer_id();

        // Return `true` if the peer is new, otherwise `false`.
        self.write_inner().peers.insert(peer_id.clone(), peer).is_none()
    }

    fn remove_peer(&self, peer_id: &PeerId) -> bool {
        // Return `false` if the peer didn't need removing, otherwise `true`.
        self.write_inner().peers.remove(peer_id).is_some()
    }

    fn get_peer(&self, peer_id: &PeerId) -> Option<Peer> {
        self.read_inner().peers.get(peer_id).map(|p| p.clone())
    }

    fn get_peers(&self) -> Vec<Peer> {
        self.read_inner().peers.values().cloned().collect::<Vec<Peer>>()
    }

    fn last_verification_request(&self, peer_id: &PeerId) -> Option<Timestamp> {
        self.read_inner().last_verif_requests.get(peer_id).copied()
    }

    fn last_verification_response(&self, peer_id: &PeerId) -> Option<Timestamp> {
        self.read_inner().last_verif_responses.get(peer_id).copied()
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

    fn update_peer_metrics(&self, peer_id: &PeerId, f: impl Fn(&mut PeerMetrics)) {
        if let Some(metrics) = self.write_inner().metrics.get_mut(peer_id) {
            f(metrics);
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

    fn insert_peer(&self, peer: Peer) -> bool {
        todo!()
    }

    fn remove_peer(&self, peer_id: &PeerId) -> bool {
        todo!()
    }

    fn get_peer(&self, peer_id: &PeerId) -> Option<Peer> {
        todo!()
    }

    fn get_peers(&self) -> Vec<Peer> {
        todo!()
    }

    fn last_verification_request(&self, peer_id: &PeerId) -> Option<Timestamp> {
        todo!()
    }

    fn last_verification_response(&self, peer_id: &PeerId) -> Option<Timestamp> {
        todo!()
    }

    fn update_last_verification_request(&self, peer_id: PeerId) {
        todo!()
    }
    fn update_last_verification_response(&self, peer_id: PeerId) {
        todo!()
    }

    fn update_peer_metrics(&self, peer_id: &PeerId, f: impl Fn(&mut PeerMetrics)) {
        todo!()
    }
}
