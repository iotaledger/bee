// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::peer::Peer;

use bee_network::{MessageSender, PeerId};

use futures::channel::oneshot;
use log::debug;
use tokio::sync::{RwLock, RwLockReadGuard};

use std::{collections::HashMap, sync::Arc};

pub struct PeerManager {
    // TODO private
    pub(crate) peers: RwLock<HashMap<PeerId, (Arc<Peer>, Option<(MessageSender, oneshot::Sender<()>)>)>>,
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

    pub async fn is_empty(&self) -> bool {
        self.peers.read().await.is_empty()
    }

    // TODO find a way to only return a ref to the peer.
    pub async fn get(
        &self,
        id: &PeerId,
    ) -> Option<impl std::ops::Deref<Target = (Arc<Peer>, Option<(MessageSender, oneshot::Sender<()>)>)> + '_> {
        RwLockReadGuard::try_map(self.peers.read().await, |map| map.get(id)).ok()
    }

    pub async fn get_all(&self) -> Vec<Arc<Peer>> {
        let mut ret = Vec::new();
        for (_, (peer, _)) in self.peers.read().await.iter() {
            ret.push(peer.clone());
        }
        ret
    }

    // // TODO find a way to only return a ref to the peer.
    // TODO implement
    // pub(crate) async fn get_mut(
    //     &self,
    //     id: &PeerId,
    // ) -> Option<impl std::ops::DerefMut<Target = (Arc<Peer>, Option<(MessageSender, oneshot::Sender<()>)>)> + '_> {
    //     RwLockWriteGuard::try_map(self.peers.write().await, |map| map.get(id)).ok()
    // }

    pub(crate) async fn add(&self, peer: Arc<Peer>) {
        debug!("Added peer {}.", peer.id());
        self.peers_keys.write().await.push(peer.id().clone());
        self.peers.write().await.insert(peer.id().clone(), (peer, None));
    }

    pub(crate) async fn remove(
        &self,
        id: &PeerId,
    ) -> Option<(Arc<Peer>, Option<(MessageSender, oneshot::Sender<()>)>)> {
        debug!("Removed peer {}.", id);
        self.peers_keys.write().await.retain(|peer_id| peer_id != id);
        self.peers.write().await.remove(id)
    }

    // TODO bring it back
    // pub(crate) async fn for_each_peer<F: Fn(&PeerId, &Peer)>(&self, f: F) {
    //     for (id, (peer, _, _)) in self.peers.read().await.iter() {
    //         f(id, peer);
    //     }
    // }

    pub async fn is_connected(&self, id: &PeerId) -> bool {
        self.peers.read().await.get(id).map(|p| p.1.is_some()).unwrap_or(false)
    }

    pub async fn connected_peers(&self) -> u8 {
        let mut count = 0;

        for (_, (_, ctx)) in self.peers.read().await.iter() {
            if ctx.is_some() {
                count += 1;
            }
        }

        count
    }

    pub(crate) fn synced_peers(&self) -> u8 {
        // TODO impl
        0
    }
}
