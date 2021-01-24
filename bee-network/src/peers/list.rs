// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Multiaddr, ShortId, MAX_UNKNOWN_PEERS};

use super::{errors::Error, PeerRelation};

use libp2p::PeerId;
use log::trace;
use tokio::sync::RwLock;

use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc},
};

const DEFAULT_PEERLIST_CAPACITY: usize = 8;

#[derive(Clone, Default)]
pub struct PeerList(Arc<RwLock<HashMap<PeerId, (PeerInfo, PeerState)>>>);

impl PeerList {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::with_capacity(DEFAULT_PEERLIST_CAPACITY))))
    }

    // If the insertion fails for some reason, we give it back to the caller.
    pub async fn insert(&self, id: PeerId, info: PeerInfo, state: PeerState) -> Result<(), (PeerId, PeerInfo, Error)> {
        if let Err(e) = self.accepts(&id, &info).await {
            Err((id, info, e))
        } else {
            // Since we already checked that such an `id` is not yet present, the returned value is always `None`.
            let _ = self.0.write().await.insert(id, (info, state));
            Ok(())
        }
    }

    pub async fn update_relation(&self, id: &PeerId, relation: PeerRelation) -> Result<(), Error> {
        let mut this = self.0.write().await;
        let mut kv = this.get_mut(id).ok_or_else(|| Error::UnlistedPeer(id.short()))?;

        kv.0.relation = relation;

        Ok(())
    }

    pub async fn update_state(&self, id: &PeerId, state: PeerState) -> Result<(), Error> {
        let mut this = self.0.write().await;
        let mut kv = this.get_mut(id).ok_or_else(|| Error::UnlistedPeer(id.short()))?;

        kv.1 = state;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn contains(&self, id: &PeerId) -> bool {
        self.0.read().await.contains_key(id)
    }

    pub async fn accepts(&self, id: &PeerId, info: &PeerInfo) -> Result<(), Error> {
        if self.0.read().await.contains_key(id) {
            let alias = info.alias.clone();
            return Err(Error::PeerAlreadyAdded(alias));
        }

        // Prevent inserting more peers than preconfigured.
        match info.relation {
            PeerRelation::Unknown => {
                if self.count_if(|info, _| info.relation.is_unknown()).await
                    >= MAX_UNKNOWN_PEERS.load(Ordering::Relaxed)
                {
                    return Err(Error::UnknownPeerLimitReached(
                        MAX_UNKNOWN_PEERS.load(Ordering::Relaxed),
                    ));
                }
            }
            // TODO: Handle 'PeerRelation::Discovered' case once autopeering has landed.
            _ => (),
        }
        if self.0.read().await.contains_key(id) {
            let alias = info.alias.clone();
            return Err(Error::PeerAlreadyAdded(alias));
        }

        Ok(())
    }

    pub async fn remove(&self, id: &PeerId) -> Result<PeerInfo, Error> {
        let (info, _) = self
            .0
            .write()
            .await
            .remove(id)
            .ok_or_else(|| Error::UnlistedPeer(id.short()))?;

        Ok(info)
    }

    #[allow(dead_code)]
    pub async fn count(&self) -> usize {
        self.0.read().await.len()
    }

    pub async fn get_info(&self, id: &PeerId) -> Result<PeerInfo, Error> {
        self.0
            .read()
            .await
            .get(id)
            .ok_or_else(|| Error::UnlistedPeer(id.short()))
            // .map(|kv| kv.value().0.clone())
            .map(|kv| kv.0.clone())
    }

    pub async fn is(&self, id: &PeerId, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) -> Result<bool, Error> {
        self.0
            .read()
            .await
            .get(id)
            .ok_or_else(|| Error::UnlistedPeer(id.short()))
            // .map(|kv| predicate(&kv.value().0, &kv.value().1))
            .map(|kv| predicate(&kv.0, &kv.1))
    }

    pub async fn iter_if(&self, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) -> impl Iterator<Item = PeerId> {
        self.0
            .read()
            .await
            .iter()
            .filter_map(|kv| {
                let (info, state) = kv.1; // kv.value();
                if predicate(info, state) {
                    // Some(kv.key().clone())
                    Some(kv.0.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<PeerId>>()
            .into_iter()
    }

    pub async fn count_if(&self, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) -> usize {
        self.0.read().await.iter().fold(0, |count, kv| {
            // let (info, state) = kv.value();
            let (info, state) = kv.1;
            if predicate(info, state) {
                count + 1
            } else {
                count
            }
        })
    }

    pub async fn remove_if(&self, id: &PeerId, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) {
        let remove = if let Some((info, state)) = self.0.read().await.get(id) {
            predicate(info, state)
        } else {
            return;
        };

        if remove {
            let _ = self.0.write().await.remove(id);
        }
    }

    #[allow(dead_code)]
    pub async fn clear(&self) {
        trace!("Dropping message senders.");
        self.0.write().await.clear();
    }
}

/// Additional information about a peer.
#[derive(Clone, Debug)]
pub struct PeerInfo {
    /// The peer's address.
    pub address: Multiaddr,
    /// The peer's alias.
    pub alias: String,
    /// The type of relation we have with this peer.
    pub relation: PeerRelation,
}

#[derive(Clone)]
pub enum PeerState {
    Disconnected,
    Connected,
}

impl PeerState {
    pub fn is_connected(&self) -> bool {
        matches!(*self, PeerState::Connected)
    }

    pub fn is_disconnected(&self) -> bool {
        matches!(*self, PeerState::Disconnected)
    }
}
