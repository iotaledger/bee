// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{peer_id::PeerId, peerlist::PeerMetrics, Peer};

use crate::{
    delay::DelayFactory,
    local::service_map::AUTOPEERING_SERVICE_NAME,
    packet::{MessageType, OutgoingPacket},
    request::RequestManager,
    server::ServerTx,
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

    fn store_peer(&self, peer: Peer) -> bool;
    fn store_metrics(&self, peer_id: PeerId, metrics: PeerMetrics);

    fn fetch_peer(&self, peer_id: &PeerId) -> Option<Peer>;
    fn fetch_peers(&self) -> Vec<Peer>;
    fn fetch_metrics(&self, peer_id: &PeerId) -> Option<PeerMetrics>;
    fn fetch_last_verification_request(&self, peer_id: &PeerId) -> Option<Timestamp>;
    fn fetch_last_verification_response(&self, peer_id: &PeerId) -> Option<Timestamp>;

    fn update_last_verification_request(&self, peer_id: PeerId);
    fn update_last_verification_response(&self, peer_id: PeerId);

    fn delete_peer(&self, peer_id: &PeerId) -> bool;
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

    fn store_peer(&self, peer: Peer) -> bool {
        let peer_id = peer.peer_id();

        // Return `true` if the peer is new, otherwise `false`.
        self.write().peers.insert(peer_id.clone(), peer).is_none()
    }

    fn delete_peer(&self, peer_id: &PeerId) -> bool {
        // Return `false` if the peer didn't need removing, otherwise `true`.
        self.write().peers.remove(peer_id).is_some()
    }

    fn fetch_peer(&self, peer_id: &PeerId) -> Option<Peer> {
        self.read().peers.get(peer_id).map(|p| p.clone())
    }

    fn fetch_peers(&self) -> Vec<Peer> {
        self.read().peers.values().cloned().collect::<Vec<Peer>>()
    }

    fn fetch_last_verification_request(&self, peer_id: &PeerId) -> Option<Timestamp> {
        self.read().last_verif_requests.get(peer_id).copied()
    }

    fn fetch_last_verification_response(&self, peer_id: &PeerId) -> Option<Timestamp> {
        self.read().last_verif_responses.get(peer_id).copied()
    }

    fn update_last_verification_request(&self, peer_id: PeerId) {
        let _ = self.write().last_verif_requests.insert(peer_id, time::unix_now_secs());
    }

    fn update_last_verification_response(&self, peer_id: PeerId) {
        let _ = self.write().last_verif_responses.insert(peer_id, time::unix_now_secs());
    }

    fn store_metrics(&self, peer_id: PeerId, peer_metrics: PeerMetrics) {
        self.write().metrics.insert(peer_id, peer_metrics);
    }

    fn fetch_metrics(&self, peer_id: &PeerId) -> Option<PeerMetrics> {
        self.read().metrics.get(peer_id).cloned()
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

    fn store_peer(&self, peer: Peer) -> bool {
        todo!()
    }

    fn delete_peer(&self, peer_id: &PeerId) -> bool {
        todo!()
    }

    fn fetch_peer(&self, peer_id: &PeerId) -> Option<Peer> {
        todo!()
    }

    fn fetch_peers(&self) -> Vec<Peer> {
        todo!()
    }

    fn fetch_last_verification_request(&self, peer_id: &PeerId) -> Option<Timestamp> {
        todo!()
    }

    fn fetch_last_verification_response(&self, peer_id: &PeerId) -> Option<Timestamp> {
        todo!()
    }

    fn update_last_verification_request(&self, peer_id: PeerId) {
        todo!()
    }
    fn update_last_verification_response(&self, peer_id: PeerId) {
        todo!()
    }

    fn store_metrics(&self, peer_id: PeerId, peer_metrics: PeerMetrics) {
        todo!()
    }

    fn fetch_metrics(&self, peer_id: &PeerId) -> Option<PeerMetrics> {
        todo!()
    }
}
