// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Multiaddr, ShortId, KNOWN_PEER_LIMIT, UNKNOWN_PEER_LIMIT};

use super::{errors::Error, DataSender, PeerRelation};

use dashmap::DashMap;
use libp2p::PeerId;

use std::sync::{atomic::Ordering, Arc};

const DEFAULT_PEERLIST_CAPACITY: usize = 8;

#[derive(Clone, Debug, Default)]
pub struct PeerList(Arc<DashMap<PeerId, (PeerInfo, PeerState)>>);

impl PeerList {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::with_capacity(DEFAULT_PEERLIST_CAPACITY)))
    }

    // If the insertion fails for some reason, we give it back to the caller.
    pub fn insert(&self, id: PeerId, info: PeerInfo, state: PeerState) -> Result<(), (PeerId, PeerInfo, Error)> {
        if self.0.contains_key(&id) {
            let short = id.short();
            return Err((id, info, Error::PeerAlreadyAdded(short)));
        }

        // Prevent inserting more peers than preconfigured.
        match info.relation {
            PeerRelation::Known => {
                if self.count_if(|info, _| info.is_known()) >= KNOWN_PEER_LIMIT.load(Ordering::Relaxed) {
                    return Err((
                        id,
                        info,
                        Error::KnownPeerLimitReached(KNOWN_PEER_LIMIT.load(Ordering::Relaxed)),
                    ));
                }
            }
            PeerRelation::Unknown => {
                if self.count_if(|info, _| info.is_unknown()) >= UNKNOWN_PEER_LIMIT.load(Ordering::Relaxed) {
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
        let _ = self.0.insert(id, (info, state));

        Ok(())
    }

    pub fn update_relation(&self, id: &PeerId, relation: PeerRelation) -> Result<(), Error> {
        let mut kv = self.0.get_mut(id).ok_or(Error::UnlistedPeer(id.short()))?;

        kv.value_mut().0.relation = relation;

        Ok(())
    }

    pub fn update_state(&self, id: &PeerId, state: PeerState) -> Result<(), Error> {
        let mut kv = self.0.get_mut(id).ok_or(Error::UnlistedPeer(id.short()))?;

        kv.value_mut().1 = state;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn contains(&self, id: &PeerId) -> bool {
        self.0.contains_key(id)
    }

    pub fn remove(&self, id: &PeerId) -> Result<PeerInfo, Error> {
        let (_, (peer_info, _)) = self.0.remove(id).ok_or(Error::UnlistedPeer(id.short()))?;

        Ok(peer_info)
    }

    pub async fn send_message(&self, message: Vec<u8>, to: &PeerId) -> Result<(), Error> {
        let mut kv = self.0.get_mut(to).ok_or(Error::UnlistedPeer(to.short()))?;

        let state = &mut kv.value_mut().1;

        if let PeerState::Connected(sender) = state {
            sender
                .send_async(message)
                .await
                .map_err(|_| Error::SendMessageFailure(to.short()))?;

            Ok(())
        } else {
            Err(Error::DisconnectedPeer(to.short()))
        }
    }

    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.0.len()
    }

    pub fn get_info(&self, id: &PeerId) -> Result<PeerInfo, Error> {
        self.0
            .get(id)
            .ok_or(Error::UnlistedPeer(id.short()))
            .map(|kv| kv.value().0.clone())
    }

    pub fn is(&self, id: &PeerId, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) -> Result<bool, Error> {
        self.0
            .get(id)
            .ok_or(Error::UnlistedPeer(id.short()))
            .map(|kv| predicate(&kv.value().0, &kv.value().1))
    }

    pub fn iter_if(&self, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) -> impl Iterator<Item = PeerId> {
        self.0
            .iter()
            .filter_map(|kv| {
                let (info, state) = kv.value();
                if predicate(info, state) {
                    Some(kv.key().clone())
                } else {
                    None
                }
            })
            .collect::<Vec<PeerId>>()
            .into_iter()
    }

    pub fn count_if(&self, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) -> usize {
        self.0.iter().fold(0, |count, kv| {
            let (info, state) = kv.value();
            if predicate(info, state) {
                count + 1
            } else {
                count
            }
        })
    }

    pub fn remove_if(&self, id: &PeerId, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) {
        let _ = self.0.remove_if(id, |_, (info, state)| predicate(info, state));
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

#[derive(Clone, Debug)]
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
