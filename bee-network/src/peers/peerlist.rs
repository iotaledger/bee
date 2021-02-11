// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::MAX_UNKNOWN_PEERS;

use libp2p::PeerId;
use log::trace;
use tokio::sync::RwLock;

use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc},
};

use super::{Error, PeerInfo, PeerState};

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

    pub async fn upgrade_relation(&self, id: &PeerId) -> Result<(), Error> {
        let mut this = self.0.write().await;
        let kv = this.get_mut(id).ok_or_else(|| Error::UnregisteredPeer(*id))?;

        kv.0.relation.upgrade();

        Ok(())
    }

    pub async fn downgrade_relation(&self, id: &PeerId) -> Result<(), Error> {
        let mut this = self.0.write().await;
        let kv = this.get_mut(id).ok_or_else(|| Error::UnregisteredPeer(*id))?;

        kv.0.relation.downgrade();

        Ok(())
    }

    pub async fn update_state(&self, id: &PeerId, state: PeerState) -> Result<(), Error> {
        let mut this = self.0.write().await;
        let mut kv = this.get_mut(id).ok_or_else(|| Error::UnregisteredPeer(*id))?;

        kv.1 = state;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn contains(&self, id: &PeerId) -> bool {
        self.0.read().await.contains_key(id)
    }

    pub async fn accepts(&self, id: &PeerId, info: &PeerInfo) -> Result<(), Error> {
        if self.0.read().await.contains_key(id) {
            return Err(Error::PeerAlreadyAdded(*id));
        }

        // Prevent inserting more peers than preconfigured.
        if info.relation.is_unknown() {
            if self.count_if(|info, _| info.relation.is_unknown()).await >= MAX_UNKNOWN_PEERS.load(Ordering::Relaxed) {
                return Err(Error::UnknownPeerLimitReached(
                    MAX_UNKNOWN_PEERS.load(Ordering::Relaxed),
                ));
            }
        }
        if self.0.read().await.contains_key(id) {
            return Err(Error::PeerAlreadyAdded(*id));
        }

        Ok(())
    }

    pub async fn remove(&self, id: &PeerId) -> Result<PeerInfo, Error> {
        let (info, _) = self
            .0
            .write()
            .await
            .remove(id)
            .ok_or_else(|| Error::UnregisteredPeer(*id))?;

        Ok(info)
    }

    #[allow(dead_code)]
    pub async fn count(&self) -> usize {
        self.0.read().await.len()
    }

    // TODO: change return value to `Option<PeerInfo>`1
    pub async fn get_info(&self, id: &PeerId) -> Result<PeerInfo, Error> {
        self.0
            .read()
            .await
            .get(id)
            .ok_or_else(|| Error::UnregisteredPeer(*id))
            .map(|kv| kv.0.clone())
    }

    pub async fn is(&self, id: &PeerId, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) -> Result<bool, Error> {
        self.0
            .read()
            .await
            .get(id)
            .ok_or_else(|| Error::UnregisteredPeer(*id))
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
