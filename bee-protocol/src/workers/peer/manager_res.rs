// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// TODO This exist to avoid a cyclic dependency, there has to be another way.

use crate::types::peer::Peer;

use bee_network::{GossipSender, PeerId};
use bee_runtime::{node::Node, worker::Worker};

use async_trait::async_trait;
use futures::channel::oneshot;
use log::debug;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use std::{collections::HashMap, convert::Infallible, sync::Arc};

#[deprecated]
pub struct PeerManagerResWorker {}

#[async_trait]
impl<N: Node> Worker<N> for PeerManagerResWorker {
    type Config = ();
    type Error = Infallible;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        node.register_resource(PeerManager::new());

        Ok(Self {})
    }

    async fn stop(self, node: &mut N) -> Result<(), Self::Error> {
        if let Some(peer_manager) = node.remove_resource::<PeerManager>() {
            for (_, (_, sender)) in peer_manager.0.into_inner().peers {
                if let Some(sender) = sender {
                    // TODO: Should we handle this error?
                    let _ = sender.1.send(());
                }
            }
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct PeerManagerInner {
    // TODO private
    #[allow(clippy::type_complexity)] // TODO
    pub(crate) peers: HashMap<PeerId, (Arc<Peer>, Option<(GossipSender, oneshot::Sender<()>)>)>,
    // This is needed to ensure message distribution fairness as iterating over a HashMap is random.
    pub(crate) keys: Vec<PeerId>,
}

#[derive(Default)]
// TODO private
pub struct PeerManager(pub(crate) RwLock<PeerManagerInner>);

impl PeerManager {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub async fn is_empty(&self) -> bool {
        self.0.read().await.peers.is_empty()
    }

    // TODO find a way to only return a ref to the peer.
    pub async fn get(
        &self,
        id: &PeerId,
    ) -> Option<impl std::ops::Deref<Target = (Arc<Peer>, Option<(GossipSender, oneshot::Sender<()>)>)> + '_> {
        RwLockReadGuard::try_map(self.0.read().await, |map| map.peers.get(id)).ok()
    }

    pub async fn get_mut(
        &self,
        id: &PeerId,
    ) -> Option<impl std::ops::DerefMut<Target = (Arc<Peer>, Option<(GossipSender, oneshot::Sender<()>)>)> + '_> {
        RwLockWriteGuard::try_map(self.0.write().await, |map| map.peers.get_mut(id)).ok()
    }

    pub async fn get_all(&self) -> Vec<Arc<Peer>> {
        self.0
            .read()
            .await
            .peers
            .iter()
            .map(|(_, (peer, _))| peer)
            .cloned()
            .collect()
    }

    pub(crate) async fn add(&self, peer: Arc<Peer>) {
        debug!("Added peer {}.", peer.id());
        let mut lock = self.0.write().await;
        lock.keys.push(*peer.id());
        lock.peers.insert(*peer.id(), (peer, None));
    }

    pub(crate) async fn remove(&self, id: &PeerId) -> Option<(Arc<Peer>, Option<(GossipSender, oneshot::Sender<()>)>)> {
        debug!("Removed peer {}.", id);
        let mut lock = self.0.write().await;
        lock.keys.retain(|peer_id| peer_id != id);
        lock.peers.remove(id)
    }

    // TODO bring it back
    // pub(crate) async fn for_each_peer<F: Fn(&PeerId, &Peer)>(&self, f: F) {
    //     for (id, (peer, _, _)) in self.peers.read().await.iter() {
    //         f(id, peer);
    //     }
    // }

    pub async fn is_connected(&self, id: &PeerId) -> bool {
        self.0
            .read()
            .await
            .peers
            .get(id)
            .map(|p| p.1.is_some())
            .unwrap_or(false)
    }

    pub async fn connected_peers(&self) -> u8 {
        self.0
            .read()
            .await
            .peers
            .iter()
            .fold(0, |acc, (_, (_, ctx))| acc + ctx.is_some() as u8)
    }

    pub async fn synced_peers(&self) -> u8 {
        self.0.read().await.peers.iter().fold(0, |acc, (_, (peer, ctx))| {
            acc + (ctx.is_some() && peer.is_synced()) as u8
        })
    }
}
