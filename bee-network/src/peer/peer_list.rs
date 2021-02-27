// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{error::InsertionFailure, Error, PeerInfo, PeerState};
use crate::{swarm::protocols::gossip::GossipSender, MAX_DISCOVERED_PEERS, MAX_UNKNOWN_PEERS};

use libp2p::{Multiaddr, PeerId};
use tokio::sync::RwLock;

use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::Ordering, Arc},
};

// TODO: check whether this is the right default value when used in production.
const DEFAULT_PEERLIST_CAP: usize = 8;
const DEFAULT_BANNED_PEERS_CAP: usize = 32;
const DEFAULT_BANNED_ADDRS_CAP: usize = 16;

#[derive(Clone, Default)]
pub struct PeerList {
    peers: Arc<RwLock<HashMap<PeerId, (PeerInfo, PeerState)>>>,
    banned_peers: Arc<RwLock<HashSet<PeerId>>>,
    banned_addrs: Arc<RwLock<HashSet<Multiaddr>>>,
}

impl PeerList {
    /// Creates a new empty peer list.
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::with_capacity(DEFAULT_PEERLIST_CAP))),
            banned_peers: Arc::new(RwLock::new(HashSet::with_capacity(DEFAULT_BANNED_PEERS_CAP))),
            banned_addrs: Arc::new(RwLock::new(HashSet::with_capacity(DEFAULT_BANNED_ADDRS_CAP))),
        }
    }

    /// Inserts a peer id together with some metadata into the peer list.
    /// **Note**: If the insertion fails for some reason, we give it back to the caller.
    pub async fn insert(&self, peer_id: PeerId, peer_info: PeerInfo) -> Result<(), InsertionFailure> {
        if let Err(e) = self.accepts(&peer_id, &peer_info).await {
            Err(InsertionFailure(peer_id, peer_info, e))
        } else {
            // Since we already checked that such a `peer_id` is not yet present, the returned value is always `None`.
            let _ = self
                .peers
                .write()
                .await
                .insert(peer_id, (peer_info, PeerState::from_disconnected()));
            Ok(())
        }
    }

    /// Removes a peer id and associated metadata and peer state from the peer list.
    pub async fn remove(&self, peer_id: &PeerId) -> Result<PeerInfo, Error> {
        let (info, _) = self
            .peers
            .write()
            .await
            .remove(peer_id)
            .ok_or_else(|| Error::PeerUnrecognized(*peer_id))?;

        Ok(info)
    }

    /// Checks wether the peer list contains a given peer id.
    pub async fn contains(&self, peer_id: &PeerId) -> bool {
        self.peers.read().await.contains_key(peer_id)
    }

    /// Returns the info about a given peer.
    pub async fn info(&self, peer_id: &PeerId) -> Result<PeerInfo, Error> {
        self.peers
            .read()
            .await
            .get(peer_id)
            .ok_or_else(|| Error::PeerUnrecognized(*peer_id))
            .map(|(peer_info, _)| peer_info.clone())
    }

    /// Bans a peer.
    pub async fn ban_peer(&self, peer_id: PeerId) -> Result<(), Error> {
        // TODO: use storage to persist banned peers
        if self.banned_peers.write().await.insert(peer_id) {
            Ok(())
        } else {
            Err(Error::PeerAlreadyBanned(peer_id))
        }
    }

    /// Bans an address.
    pub async fn ban_address(&self, address: Multiaddr) -> Result<(), Error> {
        // TODO: use storage to persist banned addrs
        if self.banned_addrs.write().await.insert(address.clone()) {
            Ok(())
        } else {
            Err(Error::AddressAlreadyBanned(address))
        }
    }

    /// Unbans a peer.
    pub async fn unban_peer(&self, peer_id: &PeerId) -> Result<(), Error> {
        if self.banned_peers.write().await.remove(peer_id) {
            Ok(())
        } else {
            Err(Error::PeerAlreadyUnbanned(*peer_id))
        }
    }

    /// Unbans an address.
    pub async fn unban_address(&self, address: &Multiaddr) -> Result<(), Error> {
        if self.banned_addrs.write().await.remove(address) {
            Ok(())
        } else {
            Err(Error::AddressAlreadyUnbanned(address.clone()))
        }
    }

    /// Checks wether the peer would be accepted by the peer list. Those checks are:
    ///     1. the peer id does not already exist,
    ///     2. the `max_unknown_peers_acccepted` has not been reached yet,
    ///     3. the `max_discovered_peers_dialed` has not been reached yet,
    ///     4. the peer id has not been banned,
    ///     5. the peer address has not been banned,
    pub async fn accepts(&self, peer_id: &PeerId, peer_info: &PeerInfo) -> Result<(), Error> {
        // Deny banned peers.
        if self.banned_peers.read().await.contains(peer_id) {
            return Err(Error::PeerBanned(*peer_id));
        }

        // Deny banned addresses.
        if self.banned_addrs.read().await.contains(&peer_info.address) {
            return Err(Error::AddressBanned(peer_info.address.clone()));
        }

        // Deny duplicates.
        if self.peers.read().await.contains_key(peer_id) {
            return Err(Error::PeerAlreadyAdded(*peer_id));
        }

        // Deny more than `MAX_UNKNOWN_PEERS` peers.
        if peer_info.relation.is_unknown()
            && self.count_if(|info, _| info.relation.is_unknown()).await >= MAX_UNKNOWN_PEERS.load(Ordering::Relaxed)
        {
            return Err(Error::MaxUnknownPeersLimitExceeded(
                MAX_UNKNOWN_PEERS.load(Ordering::Relaxed),
            ));
        }

        // Deny more than `MAX_DISCOVERED_PEERS` peers.
        if peer_info.relation.is_discovered()
            && self.count_if(|info, _| info.relation.is_discovered()).await
                >= MAX_DISCOVERED_PEERS.load(Ordering::Relaxed)
        {
            return Err(Error::MaxDiscoveredPeersLimitExceeded(
                MAX_DISCOVERED_PEERS.load(Ordering::Relaxed),
            ));
        }

        Ok(())
    }

    /// Returns the number of peers registered with the peer list.
    pub async fn len(&self) -> usize {
        self.peers.read().await.len()
    }

    /// Updates the metadata or the state associated with a given peer.
    pub async fn update_info<U>(&self, peer_id: &PeerId, mut update: U) -> Result<(), Error>
    where
        U: FnMut(&mut PeerInfo),
    {
        let mut this = self.peers.write().await;
        let (info, _) = this.get_mut(peer_id).ok_or_else(|| Error::PeerUnrecognized(*peer_id))?;

        update(info);

        Ok(())
    }

    /// Updates the metadata or the state associated with a given peer.
    pub async fn update_state<U>(&self, peer_id: &PeerId, mut update: U) -> Result<Option<GossipSender>, Error>
    where
        U: FnMut(&mut PeerState) -> Option<GossipSender>,
    {
        let mut this = self.peers.write().await;
        let (_, state) = this.get_mut(peer_id).ok_or_else(|| Error::PeerUnrecognized(*peer_id))?;

        Ok(update(state))
    }

    /// Checks wether a predicate is satisfied for a given peer.
    pub async fn is<P>(&self, peer_id: &PeerId, predicate: P) -> Result<bool, Error>
    where
        P: Fn(&PeerInfo, &PeerState) -> bool,
    {
        self.peers
            .read()
            .await
            .get(peer_id)
            .ok_or_else(|| Error::PeerUnrecognized(*peer_id))
            .map(|(info, state)| predicate(info, state))
    }

    /// Returns an iterator over peers that satisfy a given predicate.
    pub async fn iter_if<P>(&self, predicate: P) -> impl Iterator<Item = (PeerId, String)>
    where
        P: Fn(&PeerInfo, &PeerState) -> bool,
    {
        self.peers
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

    /// Counts the number of peers that satisfy a given predicate.
    pub async fn count_if<P>(&self, predicate: P) -> usize
    where
        P: Fn(&PeerInfo, &PeerState) -> bool,
    {
        self.peers.read().await.iter().fold(
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

    /// Removes a peer if it satisfies a given predicate.
    pub async fn remove_if<P>(&self, peer_id: &PeerId, predicate: P) -> bool
    where
        P: Fn(&PeerInfo, &PeerState) -> bool,
    {
        // **NB**: We need to be very cautious here to not accidentally nest the requests for the lock! Note, that
        // we would then still hold references (`info` and `state`) into the datastructure while waiting for the write
        // handle.

        let can_remove = if let Some((info, state)) = self.peers.read().await.get(peer_id) {
            predicate(info, state)
        } else {
            false
        };

        if can_remove {
            self.peers.write().await.remove(peer_id).is_some()
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub async fn clear(&self) {
        self.peers.write().await.clear();
    }
}
