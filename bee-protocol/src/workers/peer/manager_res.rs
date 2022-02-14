// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// TODO This exist to avoid a cyclic dependency, there has to be another way.

use crate::types::peer::Peer;

use bee_gossip::{GossipSender, PeerId};
use bee_runtime::{node::Node, worker::Worker};

use async_trait::async_trait;
use futures::channel::oneshot;
use log::debug;
use parking_lot::RwLock;

use std::{
    convert::Infallible,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

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
            for (_, (_, sender)) in peer_manager.inner.into_inner().peers {
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
struct PeerManagerInner {
    peers: Vec<(PeerId, PeerTuple)>,
}

impl PeerManagerInner {
    fn get(&self, id: &PeerId) -> Option<&PeerTuple> {
        self.peers
            .binary_search_by_key(id, |(id, _)| *id)
            .ok()
            .map(|i| &self.peers[i].1)
    }

    fn get_mut(&mut self, id: &PeerId) -> Option<&mut PeerTuple> {
        self.peers
            .binary_search_by_key(id, |(id, _)| *id)
            .ok()
            .map(|i| &mut self.peers[i].1)
    }

    fn insert(&mut self, id: PeerId, peer: PeerTuple) {
        match self.peers.binary_search_by_key(&id, |(id, _)| *id) {
            Ok(i) => self.peers[i] = (id, peer),
            Err(i) => self.peers.insert(i, (id, peer)),
        }
    }

    fn remove(&mut self, id: &PeerId) -> Option<PeerTuple> {
        if let Ok(i) = self.peers.binary_search_by_key(id, |(id, _)| *id) {
            Some(self.peers.remove(i).1)
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct PeerManager {
    inner: RwLock<PeerManagerInner>,
    counter: AtomicUsize,
}

impl PeerManager {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.read().peers.is_empty()
    }

    pub fn get_map<T>(&self, id: &PeerId, f: impl Fn(&PeerTuple) -> T) -> Option<T> {
        let guard = self.inner.read();
        let output = guard.get(id).map(f);
        drop(guard);
        output
    }

    pub fn get_mut_map<T>(&self, id: &PeerId, f: impl FnOnce(&mut PeerTuple) -> T) -> Option<T> {
        let mut guard = self.inner.write();
        let output = guard.get_mut(id).map(f);
        drop(guard);
        output
    }

    pub fn get_all(&self) -> Vec<Arc<Peer>> {
        self.inner
            .read()
            .peers
            .iter()
            .map(|(_, (peer, _))| peer)
            .cloned()
            .collect()
    }

    pub(crate) fn add(&self, peer: Arc<Peer>) {
        debug!("Added peer {}.", peer.id());
        let mut lock = self.inner.write();
        lock.insert(*peer.id(), (peer, None));
    }

    pub(crate) fn remove(&self, id: &PeerId) -> Option<PeerTuple> {
        debug!("Removed peer {}.", id);
        let mut lock = self.inner.write();
        lock.remove(id)
    }

    pub(crate) fn for_each<F: Fn(&PeerId, &Peer)>(&self, f: F) {
        self.inner.read().peers.iter().for_each(|(id, (peer, _))| f(id, peer));
    }

    /// Find one peer that satisfies a condition. If more than one peer satisfies this condition,
    /// each peer is equally likely to be returned.
    pub(crate) fn fair_find(&self, f: impl Fn(&Peer) -> bool) -> Option<PeerId> {
        let guard = self.inner.read();

        for _ in 0..guard.peers.len() {
            let counter = self.counter.fetch_add(1, Ordering::Relaxed);
            let (peer_id, (peer, _)) = &guard.peers[counter % guard.peers.len()];

            if f(peer.as_ref()) {
                return Some(*peer_id);
            }
        }

        drop(guard);

        None
    }

    pub fn is_connected(&self, id: &PeerId) -> bool {
        self.inner.read().get(id).map_or(false, |p| p.1.is_some())
    }

    pub fn connected_peers(&self) -> u8 {
        self.inner
            .read()
            .peers
            .iter()
            .filter(|(_, (_, ctx))| ctx.is_some())
            .count() as u8
    }

    pub fn synced_peers(&self) -> u8 {
        self.inner
            .read()
            .peers
            .iter()
            .filter(|(_, (peer, ctx))| (ctx.is_some() && peer.is_synced()))
            .count() as u8
    }
}
