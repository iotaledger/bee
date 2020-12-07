// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Multiaddr, ShortId, KNOWN_PEER_LIMIT, UNKNOWN_PEER_LIMIT};

use super::{errors::Error, DataSender, PeerRelation};

use libp2p::PeerId;
use tokio::sync::RwLock;

use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc},
};

const DEFAULT_PEERLIST_CAPACITY: usize = 8;

// #[derive(Clone, Debug, Default)]
// pub struct PeerList(Arc<DashMap<PeerId, (PeerInfo, PeerState)>>);

#[derive(Clone, Default)]
pub struct PeerList(Arc<RwLock<HashMap<PeerId, (PeerInfo, PeerState)>>>);

impl PeerList {
    pub fn new() -> Self {
        // Self(Arc::new(DashMap::with_capacity(DEFAULT_PEERLIST_CAPACITY)))
        Self(Arc::new(RwLock::new(HashMap::with_capacity(DEFAULT_PEERLIST_CAPACITY))))
    }

    // If the insertion fails for some reason, we give it back to the caller.
    pub async fn insert(&self, id: PeerId, info: PeerInfo, state: PeerState) -> Result<(), (PeerId, PeerInfo, Error)> {
        // TODO: unwrap
        if self.0.read().await.contains_key(&id) {
            let short = id.short();
            return Err((id, info, Error::PeerAlreadyAdded(short)));
        }

        // Prevent inserting more peers than preconfigured.
        match info.relation {
            PeerRelation::Known => {
                if self.count_if(|info, _| info.is_known()).await >= KNOWN_PEER_LIMIT.load(Ordering::Relaxed) {
                    return Err((
                        id,
                        info,
                        Error::KnownPeerLimitReached(KNOWN_PEER_LIMIT.load(Ordering::Relaxed)),
                    ));
                }
            }
            PeerRelation::Unknown => {
                if self.count_if(|info, _| info.is_unknown()).await >= UNKNOWN_PEER_LIMIT.load(Ordering::Relaxed) {
                    return Err((
                        id,
                        info,
                        Error::UnknownPeerLimitReached(UNKNOWN_PEER_LIMIT.load(Ordering::Relaxed)),
                    ));
                }
            }
            PeerRelation::Discovered => {
                todo!("PeerRelation::Discovered case");
            }
        }

        // Since we already checked that such an `id` is not yet present, the returned value is always `None`.
        // TODO: unwrap
        let _ = self.0.write().await.insert(id, (info, state));

        Ok(())
    }

    pub async fn update_relation(&self, id: &PeerId, relation: PeerRelation) -> Result<(), Error> {
        // TODO: unwrap
        let mut this = self.0.write().await;
        let mut kv = this.get_mut(id).ok_or(Error::UnlistedPeer(id.short()))?;

        // kv.value_mut().0.relation = relation;
        kv.0.relation = relation;

        Ok(())
    }

    pub async fn update_state(&self, id: &PeerId, state: PeerState) -> Result<(), Error> {
        let mut this = self.0.write().await;
        let mut kv = this.get_mut(id).ok_or(Error::UnlistedPeer(id.short()))?;

        // kv.value_mut().1 = state;
        kv.1 = state;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn contains(&self, id: &PeerId) -> bool {
        self.0.read().await.contains_key(id)
    }

    pub async fn remove(&self, id: &PeerId) -> Result<PeerInfo, Error> {
        let (info, _) = self.0.write().await.remove(id).ok_or(Error::UnlistedPeer(id.short()))?;

        Ok(info)
    }

    // TODO: batch messages before sending using 'send_all' (e.g. batch messages for like 50ms)
    pub async fn send_message(&self, message: Vec<u8>, to: &PeerId) -> Result<(), Error> {
        // TODO: unwrap
        let mut this = self.0.write().await;
        let (_, state) = this.get_mut(to).ok_or(Error::UnlistedPeer(to.short()))?;

        // let state = &mut kv.value_mut().1;
        // let state = &mut kv.1;

        if let PeerState::Connected(sender) = state {
            sender
                .send(message)
                // .unbounded_send(message)
                // NOTE: this has lifetime consequence for 'this'
                // .send_async(message)
                // .await
                .map_err(|_| Error::SendMessageFailure(to.short()))?;

            Ok(())
        } else {
            Err(Error::DisconnectedPeer(to.short()))
        }
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
            .ok_or(Error::UnlistedPeer(id.short()))
            // .map(|kv| kv.value().0.clone())
            .map(|kv| kv.0.clone())
    }

    pub async fn is(&self, id: &PeerId, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) -> Result<bool, Error> {
        self.0
            .read()
            .await
            .get(id)
            .ok_or(Error::UnlistedPeer(id.short()))
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
            // .remove_if(id, |_, (info, state)| predicate(info, state));
        }
    }

    pub async fn clear(&self) {
        self.0.write().await.clear();
    }
}

#[derive(Clone, Debug)]
pub struct PeerInfo {
    pub address: Multiaddr,
    pub alias: Option<String>,
    pub relation: PeerRelation,
}

macro_rules! impl_relation_iter {
    ($is:tt) => {
        impl PeerInfo {
            pub fn $is(&self) -> bool {
                self.relation.$is()
            }
        }
    };
}

impl_relation_iter!(is_known);
impl_relation_iter!(is_unknown);
impl_relation_iter!(is_discovered);

#[derive(Clone)]
pub enum PeerState {
    Disconnected,
    Connected(DataSender),
}

impl PeerState {
    pub fn is_connected(&self) -> bool {
        if let PeerState::Connected(_) = *self {
            true
        } else {
            false
        }
    }

    pub fn is_disconnected(&self) -> bool {
        if let PeerState::Disconnected = *self {
            true
        } else {
            false
        }
    }
}
