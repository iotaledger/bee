// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// TODO: Add banned PeerIds and Addresses from the config file.

use super::{error::Error, meta::PeerState};

use crate::{
    alias,
    config::Peer,
    init::global::max_unknown_peers,
    swarm::protocols::gossip::GossipSender,
    types::{PeerInfo, PeerRelation},
};

use libp2p::{Multiaddr, PeerId};

use std::collections::{HashMap, HashSet};

const REMOTE_PEERS_CAP: usize = 8;
const LOCAL_ADDRS_CAP: usize = 4;

#[derive(Debug)]
pub struct PeerList {
    local_id: PeerId,
    local_addrs: HashSet<Multiaddr>,
    peers: HashMap<PeerId, (PeerInfo, PeerState)>,
    banned_peers: HashSet<PeerId>,
    banned_addrs: HashSet<Multiaddr>,
}

impl PeerList {
    pub fn new(local_id: PeerId) -> Self {
        Self {
            local_id,
            local_addrs: HashSet::with_capacity(LOCAL_ADDRS_CAP),
            peers: HashMap::with_capacity(REMOTE_PEERS_CAP),
            banned_peers: HashSet::default(),
            banned_addrs: HashSet::default(),
        }
    }

    pub fn from_peers(local_id: PeerId, peers: Vec<Peer>) -> Self {
        let mut p = HashMap::with_capacity(REMOTE_PEERS_CAP);

        p.extend(peers.into_iter().map(|peer| {
            (
                peer.peer_id,
                (
                    PeerInfo {
                        address: peer.multiaddr,
                        alias: peer.alias.unwrap_or(alias!(peer.peer_id).to_owned()),
                        relation: PeerRelation::Known,
                    },
                    PeerState::new_disconnected(),
                ),
            )
        }));

        Self {
            local_id,
            local_addrs: HashSet::with_capacity(LOCAL_ADDRS_CAP),
            peers: p,
            banned_peers: HashSet::default(),
            banned_addrs: HashSet::default(),
        }
    }

    pub fn insert_peer(&mut self, peer_id: PeerId, peer_info: PeerInfo) -> Result<(), (PeerId, PeerInfo, Error)> {
        // if let Err(e) = self.accepts_inserting_peer(&peer_id, &peer_info) {
        //     return Err((peer_id, peer_info, e));
        // }

        // Since we already checked that such a `peer_id` is not yet present, the returned value is always `None`.
        let _ = self.peers.insert(peer_id, (peer_info, PeerState::new_disconnected()));

        Ok(())
    }

    pub fn insert_local_addr(&mut self, addr: Multiaddr) -> Result<(), (Multiaddr, Error)> {
        if self.local_addrs.contains(&addr) {
            return Err((addr.clone(), Error::AddressIsAdded(addr)));
        }

        let _ = self.local_addrs.insert(addr);

        Ok(())
    }

    pub fn remove(&mut self, peer_id: &PeerId) -> Result<PeerInfo, Error> {
        let (info, _) = self
            .peers
            .remove(peer_id)
            .ok_or_else(|| Error::PeerNotPresent(*peer_id))?;

        Ok(info)
    }

    pub fn contains(&self, peer_id: &PeerId) -> bool {
        self.peers.contains_key(peer_id)
    }

    pub fn info(&self, peer_id: &PeerId) -> Result<PeerInfo, Error> {
        self.peers
            .get(peer_id)
            .ok_or_else(|| Error::PeerNotPresent(*peer_id))
            .map(|(peer_info, _)| peer_info.clone())
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }

    pub fn update_info<U>(&mut self, peer_id: &PeerId, mut update: U) -> Result<(), Error>
    where
        U: FnMut(&mut PeerInfo),
    {
        let (info, _) = self
            .peers
            .get_mut(peer_id)
            .ok_or_else(|| Error::PeerNotPresent(*peer_id))?;

        update(info);

        Ok(())
    }

    pub fn update_state<U>(&mut self, peer_id: &PeerId, mut update: U) -> Result<Option<GossipSender>, Error>
    where
        U: FnMut(&mut PeerState) -> Option<GossipSender>,
    {
        let (_, state) = self
            .peers
            .get_mut(peer_id)
            .ok_or_else(|| Error::PeerNotPresent(*peer_id))?;

        Ok(update(state))
    }

    pub fn is<P>(&self, peer_id: &PeerId, predicate: P) -> Result<bool, Error>
    where
        P: Fn(&PeerInfo, &PeerState) -> bool,
    {
        self.peers
            .get(peer_id)
            .ok_or_else(|| Error::PeerNotPresent(*peer_id))
            .map(|(info, state)| predicate(info, state))
    }

    pub fn iter_if<P>(&self, predicate: P) -> impl Iterator<Item = (PeerId, String)>
    where
        P: Fn(&PeerInfo, &PeerState) -> bool,
    {
        self.peers
            .iter()
            .filter_map(|(peer_id, (info, state))| {
                if predicate(info, state) {
                    Some((*peer_id, info.alias.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn count_if<P>(&self, predicate: P) -> usize
    where
        P: Fn(&PeerInfo, &PeerState) -> bool,
    {
        self.peers.iter().fold(
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

    pub fn remove_if<P>(&mut self, peer_id: &PeerId, predicate: P) -> bool
    where
        P: Fn(&PeerInfo, &PeerState) -> bool,
    {
        let can_remove = if let Some((info, state)) = self.peers.get(peer_id) {
            predicate(info, state)
        } else {
            return false;
        };

        if can_remove {
            self.peers.remove(peer_id).is_some()
        } else {
            false
        }
    }

    #[cfg(test)]
    pub fn clear(&mut self) {
        self.peers.clear();
        self.banned_peers.clear();
        self.banned_addrs.clear();
    }

    pub fn ban_peer(&mut self, peer_id: PeerId) -> Result<(), Error> {
        // TODO: use storage to persist banned peers
        if self.banned_peers.insert(peer_id) {
            Ok(())
        } else {
            Err(Error::PeerIsBanned(peer_id))
        }
    }

    pub fn ban_address(&mut self, address: Multiaddr) -> Result<(), Error> {
        // TODO: use storage to persist banned addrs
        if self.banned_addrs.insert(address.clone()) {
            Ok(())
        } else {
            Err(Error::AddressIsBanned(address))
        }
    }

    pub fn unban_peer(&mut self, peer_id: &PeerId) -> Result<(), Error> {
        if self.banned_peers.remove(peer_id) {
            Ok(())
        } else {
            Err(Error::PeerIsUnbanned(*peer_id))
        }
    }

    pub fn unban_address(&mut self, address: &Multiaddr) -> Result<(), Error> {
        if self.banned_addrs.remove(address) {
            Ok(())
        } else {
            Err(Error::AddressIsUnbanned(address.clone()))
        }
    }

    pub fn is_peer_banned(&self, peer_id: &PeerId) -> bool {
        self.banned_peers.contains(peer_id)
    }

    pub fn is_addr_banned(&self, address: &Multiaddr) -> bool {
        self.banned_addrs.contains(address)
    }

    pub fn accepts_incoming_peer(&self, peer_id: &PeerId, peer_info: &PeerInfo) -> Result<(), Error> {
        if peer_id == &self.local_id {
            return Err(Error::PeerIsLocal(*peer_id));
        }

        if self.local_addrs.contains(&peer_info.address) {
            return Err(Error::AddressIsLocal(peer_info.address.clone()));
        }

        if self.banned_peers.contains(peer_id) {
            return Err(Error::PeerIsBanned(*peer_id));
        }

        if self.banned_addrs.contains(&peer_info.address) {
            return Err(Error::AddressIsBanned(peer_info.address.clone()));
        }

        if self.is(peer_id, |_, state| state.is_connected()).is_ok() {
            return Err(Error::PeerIsConnected(*peer_id));
        }

        if peer_info.relation.is_unknown() && self.count_if(|info, _| info.relation.is_unknown()) >= max_unknown_peers()
        {
            return Err(Error::ExceedsUnknownPeerLimit(max_unknown_peers()));
        }

        // At this point we know that the candidate is/has:
        // * not local id
        // * not local addr
        // * not banned id
        // * not banned addr
        // * not already connected
        // * not exceeding unknown peer limit

        Ok(())
    }

    pub fn allows_dialing_peer(&self, peer_id: &PeerId) -> Result<(), Error> {
        if peer_id == &self.local_id {
            return Err(Error::PeerIsLocal(*peer_id));
        }

        if !self.contains(peer_id) {
            return Err(Error::PeerNotPresent(*peer_id));
        }

        if self.banned_peers.contains(peer_id) {
            return Err(Error::PeerIsBanned(*peer_id));
        }

        if self.is(peer_id, |_, state| state.is_connected()).is_ok() {
            return Err(Error::PeerIsConnected(*peer_id));
        }

        let (peer_info, _) = self.peers.get(peer_id).unwrap();

        if self.local_addrs.contains(&peer_info.address) {
            return Err(Error::AddressIsLocal(peer_info.address.clone()));
        }

        if self.banned_addrs.contains(&peer_info.address) {
            return Err(Error::AddressIsBanned(peer_info.address.clone()));
        }

        if peer_info.relation.is_unknown() && self.count_if(|info, _| info.relation.is_unknown()) >= max_unknown_peers()
        {
            return Err(Error::ExceedsUnknownPeerLimit(max_unknown_peers()));
        }

        // At this point we know that the candidate is/has:
        // * not local id
        // * present
        // * not banned id
        // * not already connected
        // * not local addr
        // * not banned addr
        // * not exceeding unknown peer limit

        Ok(())
    }

    pub fn allows_dialing_addr(&self, addr: &Multiaddr) -> Result<(), Error> {
        if self.local_addrs.contains(addr) {
            return Err(Error::AddressIsLocal(addr.clone()));
        }

        if self.banned_addrs.contains(addr) {
            return Err(Error::AddressIsBanned(addr.clone()));
        }

        if let Some(peer_id) = self.find_peer_if_connected(addr) {
            return Err(Error::PeerIsConnected(peer_id));
        }

        // At this point we know that the candidate is/has:
        // * not local addr
        // * not banned addr
        // * not already connected to that addr

        Ok(())
    }

    fn find_peer_if_connected(&self, addr: &Multiaddr) -> Option<PeerId> {
        for (peer_id, _) in self.iter_if(|info, state| state.is_connected() && info.address == *addr) {
            return Some(peer_id);
        }

        None
    }

    // pub fn connect(&self, peer_id: &PeerId, gossip_sender: GossipSender) -> Result<(), Error> {
    //     let (_, state) = this.get_mut(peer_id).ok_or_else(|| Error::PeerMissing(*peer_id))?;

    //     if state.is_connected() {
    //         Err(Error::PeerAlreadyConnected(*peer_id))
    //     } else {
    //         state.set_connected(gossip_sender);
    //         Ok(())
    //     }
    // }

    // pub fn disconnect(&self, peer_id: &PeerId) -> Result<GossipSender, Error> {
    //     let (_, state) = this.get_mut(peer_id).ok_or_else(|| Error::PeerMissing(*peer_id))?;

    //     if state.is_disconnected() {
    //         Err(Error::PeerAlreadyDisconnected(*peer_id))
    //     } else {
    //         // `unwrap` is safe, because we know we're connected.
    //         Ok(state.set_disconnected().unwrap())
    //     }
    // }
}

// // Copyright 2020 IOTA Stiftung
// // SPDX-License-Identifier: Apache-2.0

// use super::{
//     error::{Error, InsertionFailure},
//     meta::PeerState,
// };

// use crate::{
//     alias,
//     config::Peer,
//     init::MAX_UNKNOWN_PEERS,
//     swarm::protocols::gossip::GossipSender,
//     types::{PeerInfo, PeerRelation},
// };

// use libp2p::PeerId;
// use tokio::sync::RwLock;

// use std::{collections::HashMap, sync::Arc};

// // TODO: check whether this is the right default value when used in production.
// const DEFAULT_PEERLIST_CAPACITY: usize = 8;

// // TODO: Merge with banned addr, peer_id
// // TODO: Initialize it with stuff from the config
// // TODO: Make it into global state so that we don't need to pass it around all the time
// #[derive(Clone, Default)]
// pub struct PeerList(Arc<RwLock<HashMap<PeerId, (PeerInfo, PeerState)>>>);

// impl PeerList {
//     pub fn new() -> Self {
//         Self(Arc::new(RwLock::new(HashMap::with_capacity(DEFAULT_PEERLIST_CAPACITY))))
//     }

//     pub fn from_peers(peers: Vec<Peer>) -> Self {
//         let mut m = HashMap::with_capacity(DEFAULT_PEERLIST_CAPACITY);

//         m.extend(peers.into_iter().map(|peer| {
//             (
//                 peer.peer_id,
//                 (
//                     PeerInfo {
//                         address: peer.multiaddr,
//                         alias: peer.alias.unwrap_or(alias!(peer.peer_id).to_owned()),
//                         relation: PeerRelation::Known,
//                     },
//                     PeerState::disconnected(),
//                 ),
//             )
//         }));

//         Self(Arc::new(RwLock::new(m)))
//     }

//     // If the insertion fails for some reason, we give it back to the caller.
//     pub async fn insert(&self, peer_id: PeerId, peer_info: PeerInfo) -> Result<(), InsertionFailure> {
//         if let Err(e) = self.accepts(&peer_id, &peer_info).await {
//             Err(InsertionFailure(peer_id, peer_info, e))
//         } else {
//             // Since we already checked that such a `peer_id` is not yet present, the returned value is always
// `None`.             let _ = self
//                 .0
//                 .write()
//                 .await
//                 .insert(peer_id, (peer_info, PeerState::disconnected()));
//             Ok(())
//         }
//     }

//     pub async fn upgrade_relation(&self, peer_id: &PeerId) -> Result<(), Error> {
//         let mut this = self.0.write().await;
//         let (info, _) = this.get_mut(peer_id).ok_or_else(|| Error::PeerMissing(*peer_id))?;

//         info.relation.upgrade();

//         Ok(())
//     }

//     pub async fn downgrade_relation(&self, peer_id: &PeerId) -> Result<(), Error> {
//         let mut this = self.0.write().await;
//         let (info, _) = this.get_mut(peer_id).ok_or_else(|| Error::PeerMissing(*peer_id))?;

//         info.relation.downgrade();

//         Ok(())
//     }

//     pub async fn connect(&self, peer_id: &PeerId, gossip_sender: GossipSender) -> Result<(), Error> {
//         let mut this = self.0.write().await;
//         let (_, state) = this.get_mut(peer_id).ok_or_else(|| Error::PeerMissing(*peer_id))?;

//         if state.is_connected() {
//             Err(Error::PeerAlreadyConnected(*peer_id))
//         } else {
//             state.set_connected(gossip_sender);
//             Ok(())
//         }
//     }

//     pub async fn disconnect(&self, peer_id: &PeerId) -> Result<GossipSender, Error> {
//         let mut this = self.0.write().await;
//         let (_, state) = this.get_mut(peer_id).ok_or_else(|| Error::PeerMissing(*peer_id))?;

//         if state.is_disconnected() {
//             Err(Error::PeerAlreadyDisconnected(*peer_id))
//         } else {
//             // `unwrap` is safe, because we know we're connected.
//             Ok(state.set_disconnected().unwrap())
//         }
//     }

//     pub async fn contains(&self, peer_id: &PeerId) -> bool {
//         self.0.read().await.contains_key(peer_id)
//     }

//     pub async fn accepts(&self, peer_id: &PeerId, peer_info: &PeerInfo) -> Result<(), Error> {
//         if self.0.read().await.contains_key(peer_id) {
//             return Err(Error::PeerAlreadyAdded(*peer_id));
//         }

//         // Prevent inserting more peers than preconfigured.
//         // `Unwrap`ping the global variable is fine, because we made sure that its value is set during
// initialization.         if peer_info.relation.is_unknown()
//             && self.count_if(|info, _| info.relation.is_unknown()).await >= *MAX_UNKNOWN_PEERS.get().unwrap()
//         {
//             return Err(Error::UnknownPeerLimitReached(*MAX_UNKNOWN_PEERS.get().unwrap()));
//         }
//         if self.0.read().await.contains_key(peer_id) {
//             return Err(Error::PeerAlreadyAdded(*peer_id));
//         }

//         Ok(())
//     }

//     pub async fn remove(&self, peer_id: &PeerId) -> Result<PeerInfo, Error> {
//         let (info, _) = self
//             .0
//             .write()
//             .await
//             .remove(peer_id)
//             .ok_or_else(|| Error::PeerMissing(*peer_id))?;

//         Ok(info)
//     }

//     #[allow(dead_code)]
//     pub async fn len(&self) -> usize {
//         self.0.read().await.len()
//     }

//     // TODO: change return value to `Option<PeerInfo>`1
//     pub async fn get_info(&self, peer_id: &PeerId) -> Result<PeerInfo, Error> {
//         self.0
//             .read()
//             .await
//             .get(peer_id)
//             .ok_or_else(|| Error::PeerMissing(*peer_id))
//             .map(|(peer_info, _)| peer_info.clone())
//     }

//     pub async fn update_info(&self, peer_id: &PeerId, peer_info: PeerInfo) -> Result<(), Error> {
//         let mut this = self.0.write().await;
//         let (info, _) = this.get_mut(peer_id).ok_or_else(|| Error::PeerMissing(*peer_id))?;

//         *info = peer_info;

//         Ok(())
//     }

//     pub async fn is(&self, peer_id: &PeerId, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) -> Result<bool,
// Error> {         self.0
//             .read()
//             .await
//             .get(peer_id)
//             .ok_or_else(|| Error::PeerMissing(*peer_id))
//             .map(|(info, state)| predicate(info, state))
//     }

//     pub async fn iter_if(
//         &self,
//         predicate: impl Fn(&PeerInfo, &PeerState) -> bool,
//     ) -> impl Iterator<Item = (PeerId, String)> {
//         self.0
//             .read()
//             .await
//             .iter()
//             .filter_map(|(peer_id, (info, state))| {
//                 if predicate(info, state) {
//                     Some((*peer_id, info.alias.clone()))
//                 } else {
//                     None
//                 }
//             })
//             .collect::<Vec<(PeerId, String)>>()
//             .into_iter()
//     }

//     pub async fn count_if(&self, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) -> usize {
//         self.0.read().await.iter().fold(
//             0,
//             |count, (_, (info, state))| {
//                 if predicate(info, state) { count + 1 } else { count }
//             },
//         )
//     }

//     pub async fn remove_if(&self, peer_id: &PeerId, predicate: impl Fn(&PeerInfo, &PeerState) -> bool) -> bool {
//         // NB: We need to be very cautious here to not accidentally nest the requests for the lock!

//         let can_remove = if let Some((info, state)) = self.0.read().await.get(peer_id) {
//             predicate(info, state)
//         } else {
//             false
//         };

//         if can_remove {
//             self.0.write().await.remove(peer_id).is_some()
//         } else {
//             false
//         }
//     }

//     #[allow(dead_code)]
//     pub async fn clear(&self) {
//         self.0.write().await.clear();
//     }
// }
