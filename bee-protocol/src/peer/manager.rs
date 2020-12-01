// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::peer::Peer;

use bee_network::PeerId;

use dashmap::DashMap;
use log::debug;
use tokio::sync::RwLock;

use std::sync::Arc;

pub(crate) struct PeerManager {
    pub(crate) peers: DashMap<PeerId, Arc<Peer>>,
    pub(crate) peers_keys: RwLock<Vec<PeerId>>,
}

impl PeerManager {
    pub(crate) fn new() -> Self {
        Self {
            peers: Default::default(),
            peers_keys: Default::default(),
        }
    }

    pub(crate) async fn add(&self, peer: Arc<Peer>) {
        debug!("Added peer {}.", peer.id);
        self.peers_keys.write().await.push(peer.id.clone());
        self.peers.insert(peer.id.clone(), peer);
    }

    pub(crate) async fn remove(&self, id: &PeerId) {
        debug!("Removed peer {}.", id);
        self.peers_keys.write().await.retain(|peer_id| peer_id != id);
        self.peers.remove(id);
    }

    pub(crate) fn connected_peers(&self) -> u8 {
        // TODO impl
        0
    }

    pub(crate) fn synced_peers(&self) -> u8 {
        // TODO impl
        0
    }
}
