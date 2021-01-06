// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::peer::Peer;

use bee_network::PeerId;

use dashmap::{mapref::one::Ref, DashMap};
use futures::channel::oneshot;
use log::debug;
use tokio::sync::{mpsc, RwLock};

use std::sync::Arc;

pub struct PeerManager {
    // TODO private
    pub(crate) peers: DashMap<PeerId, (Arc<Peer>, mpsc::UnboundedSender<Vec<u8>>, oneshot::Sender<()>)>,
    // This is needed to ensure message distribution fairness as iterating over a HashMap is random.
    // TODO private
    pub(crate) peers_keys: RwLock<Vec<PeerId>>,
}

impl PeerManager {
    pub(crate) fn new() -> Self {
        Self {
            peers: Default::default(),
            peers_keys: Default::default(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    // TODO find a way to only return a ref to the peer.
    pub(crate) fn get(
        &self,
        id: &PeerId,
    ) -> Option<Ref<PeerId, (Arc<Peer>, mpsc::UnboundedSender<Vec<u8>>, oneshot::Sender<()>)>> {
        // TODO this exposes Dashmap internals to avoid cloning the Arc, to revisit.
        self.peers.get(id)
    }

    pub(crate) async fn add(
        &self,
        peer: Arc<Peer>,
        sender: mpsc::UnboundedSender<Vec<u8>>,
        shutdown: oneshot::Sender<()>,
    ) {
        debug!("Added peer {}.", peer.id());
        self.peers_keys.write().await.push(peer.id().clone());
        self.peers.insert(peer.id().clone(), (peer, sender, shutdown));
    }

    pub(crate) async fn remove(
        &self,
        id: &PeerId,
    ) -> Option<(PeerId, (Arc<Peer>, mpsc::UnboundedSender<Vec<u8>>, oneshot::Sender<()>))> {
        debug!("Removed peer {}.", id);
        self.peers_keys.write().await.retain(|peer_id| peer_id != id);
        self.peers.remove(id)
    }

    pub(crate) fn for_each_peer<F: Fn(&PeerId, &Peer)>(&self, f: F) {
        for entry in self.peers.iter() {
            f(entry.key(), &*entry.value().0);
        }
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
