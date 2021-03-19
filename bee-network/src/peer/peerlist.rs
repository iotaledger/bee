// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{node::MAX_UNKNOWN_PEERS, swarm::protocols::gossip::GossipSender};

use libp2p::PeerId;
use tokio::sync::RwLock;

use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc},
};

use super::{error::InsertionFailure, Error, PeerInfo, PeerState};

// TODO: check whether this is the right default value when used in production.
const DEFAULT_PEERLIST_CAPACITY: usize = 8;

#[derive(Clone, Default)]
pub struct PeerList(Arc<RwLock<HashMap<PeerId, (PeerInfo, PeerState)>>>);

impl PeerList {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::with_capacity(DEFAULT_PEERLIST_CAPACITY))))
    }

    // If the insertion fails for some reason, we give it back to the caller.
    pub async fn insert(&self, peer_id: PeerId, peer_info: PeerInfo) -> Result<(), InsertionFailure> {
        if let Err(e) = self.accepts(&peer_id, &peer_info).await {
            Err(InsertionFailure(peer_id, peer_info, e))
        } else {
            // Since we already checked that such a `peer_id` is not yet present, the returned value is always `None`.
            let _ = self
                .0
                .write()
                .await
                .insert(peer_id, (peer_info, PeerState::disconnected()));
            Ok(())
        }
    }

    pub async fn upgrade_relation(&self, peer_id: &PeerId) -> Result<(), Error> {
        let mut this = self.0.write().await;
        let (info, _) = this.get_mut(peer_id).ok_or_else(|| Error::PeerMissing(*peer_id))?;

        info.relation.upgrade();

        Ok(())
    }

    pub async fn downgrade_relation(&self, peer_id: &PeerId) -> Result<(), Error> {
        let mut this = self.0.write().await;
        let (info, _) = this.get_mut(peer_id).ok_or_else(|| Error::PeerMissing(*peer_id))?;

        info.relation.downgrade();

        Ok(())
    }

    pub async fn connect(&self, peer_id: &PeerId, gossip_sender: GossipSender) -> Result<(), Error> {
        let mut this = self.0.write().await;
        let (_, state) = this.get_mut(peer_id).ok_or_else(|| Error::PeerMissing(*peer_id))?;

        if state.is_connected() {
            Err(Error::PeerAlreadyConnected(*peer_id))
        } else {
            state.set_connected(gossip_sender);
            Ok(())
        }
    }

    pub async fn disconnect(&self, peer_id: &PeerId) -> Result<GossipSender, Error> {
        let mut this = self.0.write().await;
        let (_, state) = this.get_mut(peer_id).ok_or_else(|| Error::PeerMissing(*peer_id))?;

        if state.is_disconnected() {
            Err(Error::PeerAlreadyDisconnected(*peer_id))
        } else {
            // `unwrap` is safe, because we know we're connected.
            Ok(state.set_disconnected().unwrap())
        }
    }

    pub async fn contains(&self, peer_id: &PeerId) -> bool {
        self.0.read().await.contains_key(peer_id)
    }

    pub async fn accepts(&self, peer_id: &PeerId, peer_info: &PeerInfo) -> Result<(), Error> {
        if self.0.read().await.contains_key(peer_id) {
            return Err(Error::PeerAlreadyAdded(*peer_id));
        }

        // Prevent inserting more peers than preconfigured.
        if peer_info.relation.is_unknown()
            && self.count_if(|info, _| info.relation.is_unknown()).await >= MAX_UNKNOWN_PEERS.load(Ordering::Relaxed)
        {
            return Err(Error::UnknownPeerLimitReached(
                MAX_UNKNOWN_PEERS.load(Ordering::Relaxed),
            ));
        }
        if self.0.read().await.contains_key(peer_id) {
            return Err(Error::PeerAlreadyAdded(*peer_id));
        }

        Ok(())
    }

    pub async fn remove(&self, peer_id: &PeerId) -> Result<PeerInfo, Error> {
        let (info, _) = self
            .0
            .write()
            .await
            .remove(peer_id)
            .ok_or_else(|| Error::PeerMissing(*peer_id))?;

        Ok(info)
    }

    #[allow(dead_code)]
    pub async fn len(&self) -> usize {
        self.0.read().await.len()
    }

    // TODO: change return value to `Option<PeerInfo>`1
    pub async fn get_info(&self, peer_id: &PeerId) -> Result<PeerInfo, Error> {
        self.0
            .read()
            .await
            .get(peer_id)
            .ok_or_else(|| Error::PeerMissing(*peer_id))
            .map(|(peer_info, _)| peer_info.clone())
    }

    pub async fn update_info(&self, peer_id: &PeerId, peer_info: PeerInfo) -> Result<(), Error> {
        let mut this = self.0.write().await;
        let (info, _) = this.get_mut(peer_id).ok_or_else(|| Error::PeerMissing(*peer_id))?;

        *info = peer_info;

        Ok(())
    }

    pub async fn is(&self, peer_id: &PeerId, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) -> Result<bool, Error> {
        self.0
            .read()
            .await
            .get(peer_id)
            .ok_or_else(|| Error::PeerMissing(*peer_id))
            .map(|(info, state)| predicate(info, state))
    }

    pub async fn iter_if(
        &self,
        predicate: impl Fn(&PeerInfo, &PeerState) -> bool,
    ) -> impl Iterator<Item = (PeerId, String)> {
        self.0
            .read()
            .await
            .iter()
            .filter_map(|(peer_id, (info, state))| {
                if predicate(info, state) {
                    Some((*peer_id, info.alias.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<(PeerId, String)>>()
            .into_iter()
    }

    pub async fn count_if(&self, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) -> usize {
        self.0.read().await.iter().fold(
            0,
            |count, (_, (info, state))| {
                if predicate(info, state) {
                    count + 1
                } else {
                    count
                }
            },
        )
    }

    pub async fn remove_if(&self, peer_id: &PeerId, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) -> bool {
        // NB: We need to be very cautious here to not accidentally nest the requests for the lock!

        let can_remove = if let Some((info, state)) = self.0.read().await.get(peer_id) {
            predicate(info, state)
        } else {
            false
        };

        if can_remove {
            self.0.write().await.remove(peer_id).is_some()
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub async fn clear(&self) {
        self.0.write().await.clear();
    }
}
