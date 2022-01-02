// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// TODO This exist to avoid a cyclic dependency, there has to be another way.

use crate::types::peer::Peer;

use bee_gossip::{GossipSender, PeerId};
use bee_runtime::{node::Node, worker::Worker};

use async_trait::async_trait;
use futures::channel::oneshot;
use log::debug;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use std::{collections::HashMap, convert::Infallible, sync::Arc};

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

type PeerTuple = (Arc<Peer>, Option<(GossipSender, oneshot::Sender<()>)>);

#[derive(Default)]
pub struct PeerManagerInner {
    peers: HashMap<PeerId, PeerTuple>,
    // This is needed to ensure message distribution fairness as iterating over a HashMap is random.
    // TODO private
    pub(crate) keys: Vec<PeerId>,
}

#[derive(Default)]
// TODO private
pub struct PeerManager(pub(crate) RwLock<PeerManagerInner>);

impl PeerManager {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.0.read().peers.is_empty()
    }

    // TODO find a way to only return a ref to the peer.
    pub fn get(&self, id: &PeerId) -> Option<impl std::ops::Deref<Target = PeerTuple> + '_> {
        RwLockReadGuard::try_map(self.0.read(), |map| map.peers.get(id)).ok()
    }

    pub fn get_mut(&self, id: &PeerId) -> Option<impl std::ops::DerefMut<Target = PeerTuple> + '_> {
        RwLockWriteGuard::try_map(self.0.write(), |map| map.peers.get_mut(id)).ok()
    }

    pub fn get_all(&self) -> Vec<Arc<Peer>> {
        self.0.read().peers.iter().map(|(_, (peer, _))| peer).cloned().collect()
    }

    pub(crate) fn add(&self, peer: Arc<Peer>) {
        debug!("Added peer {}.", peer.id());
        let mut lock = self.0.write();
        lock.keys.push(*peer.id());
        lock.peers.insert(*peer.id(), (peer, None));
    }

    pub(crate) fn remove(&self, id: &PeerId) -> Option<PeerTuple> {
        debug!("Removed peer {}.", id);
        let mut lock = self.0.write();
        lock.keys.retain(|peer_id| peer_id != id);
        lock.peers.remove(id)
    }

    pub(crate) fn for_each<F: Fn(&PeerId, &Peer)>(&self, f: F) {
        self.0.read().peers.iter().for_each(|(id, (peer, _))| f(id, peer));
    }

    pub fn is_connected(&self, id: &PeerId) -> bool {
        self.0.read().peers.get(id).map_or(false, |p| p.1.is_some())
    }

    pub fn connected_peers(&self) -> u8 {
        self.0.read().peers.iter().filter(|(_, (_, ctx))| ctx.is_some()).count() as u8
    }

    pub fn synced_peers(&self) -> u8 {
        self.0
            .read()
            .peers
            .iter()
            .filter(|(_, (peer, ctx))| (ctx.is_some() && peer.is_synced()))
            .count() as u8
    }
}
