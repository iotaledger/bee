// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{error::Error, meta::PeerState};

use crate::{
    alias,
    config::Peer,
    init::global::max_unknown_peers,
    swarm::protocols::gossip::GossipSender,
    types::{PeerInfo, PeerRelation},
};

use libp2p::{Multiaddr, PeerId};
use tokio::sync::RwLock;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

const REMOTE_PEERS_INITIAL_CAP: usize = 8;
const LOCAL_ADDRS_INITIAL_CAP: usize = 4;

/// A thread-safe wrapper around a [`PeerList`].
#[derive(Debug, Clone)]
pub struct PeerListWrapper(pub Arc<RwLock<PeerList>>);

impl PeerListWrapper {
    pub fn new(peerlist: PeerList) -> Self {
        Self(Arc::new(RwLock::new(peerlist)))
    }
}

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
            local_addrs: HashSet::with_capacity(LOCAL_ADDRS_INITIAL_CAP),
            peers: HashMap::with_capacity(REMOTE_PEERS_INITIAL_CAP),
            banned_peers: HashSet::default(),
            banned_addrs: HashSet::default(),
        }
    }

    pub fn from_peers(local_id: PeerId, peers: Vec<Peer>) -> Self {
        let mut p = HashMap::with_capacity(REMOTE_PEERS_INITIAL_CAP);

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
            local_addrs: HashSet::with_capacity(LOCAL_ADDRS_INITIAL_CAP),
            peers: p,
            banned_peers: HashSet::default(),
            banned_addrs: HashSet::default(),
        }
    }

    pub fn insert_peer(&mut self, peer_id: PeerId, peer_info: PeerInfo) -> Result<(), (PeerId, PeerInfo, Error)> {
        if self.contains(&peer_id) {
            return Err((peer_id, peer_info, Error::PeerIsAdded(peer_id)));
        }

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
                if predicate(info, state) { count + 1 } else { count }
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

        if self.is(peer_id, |_, state| state.is_connected()).unwrap_or(false) {
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

        if self.is(peer_id, |_, state| state.is_connected()).unwrap_or(false) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::{identity::ed25519::Keypair, multiaddr::Protocol};

    #[test]
    fn new_list() {
        let pl = PeerList::new(gen_constant_peer_id());

        assert_eq!(pl.len(), 0);
    }

    #[test]
    fn add_peers() {
        let local_id = gen_constant_peer_id();
        let mut pl = PeerList::new(local_id);

        for i in 1..=3 {
            assert!(
                pl.insert_peer(
                    gen_random_peer_id(),
                    gen_deterministic_peer_info(i, PeerRelation::Known)
                )
                .is_ok()
            );
            assert_eq!(pl.len(), i as usize);
        }
    }

    #[test]
    fn double_insert() {
        let local_id = gen_constant_peer_id();
        let mut pl = PeerList::new(local_id);

        let peer_id = gen_constant_peer_id();

        assert!(pl.insert_peer(peer_id, gen_constant_peer_info()).is_ok());
        assert!(matches!(
            pl.insert_peer(peer_id, gen_constant_peer_info()),
            Err((_, _, Error::PeerIsAdded(_)))
        ));
    }

    #[test]
    fn deny_incoming_local_peer() {
        let local_id = gen_constant_peer_id();

        let pl = PeerList::new(local_id);

        assert!(matches!(
            pl.accepts_incoming_peer(&local_id, &gen_constant_peer_info()),
            Err(Error::PeerIsLocal(_))
        ));
    }

    #[test]
    fn allow_incoming_added_peer() {
        let local_id = gen_constant_peer_id();
        let peer_id = gen_random_peer_id();
        let peer_info = gen_constant_peer_info();

        let mut pl = PeerList::new(local_id);

        pl.insert_peer(peer_id, peer_info.clone()).unwrap();
        pl.accepts_incoming_peer(&peer_id, &peer_info).unwrap();
    }

    // =======================================================
    // utils
    // =======================================================

    pub fn gen_constant_peer_id() -> PeerId {
        "12D3KooWJWEKvSFbben74C7H4YtKjhPMTDxd7gP7zxWSUEeF27st".parse().unwrap()
    }

    pub fn gen_random_peer_id() -> PeerId {
        PeerId::from_public_key(libp2p_core::PublicKey::Ed25519(Keypair::generate().public()))
    }

    pub fn gen_deterministic_peer_info(port: u16, relation: PeerRelation) -> PeerInfo {
        PeerInfo {
            address: gen_deterministic_addr(port),
            alias: port.to_string(),
            relation,
        }
    }

    pub fn gen_constant_peer_info() -> PeerInfo {
        PeerInfo {
            address: gen_deterministic_addr(1),
            alias: String::new(),
            relation: PeerRelation::Known,
        }
    }

    pub fn gen_deterministic_addr(port: u16) -> Multiaddr {
        let mut addr = Multiaddr::empty();
        addr.push(Protocol::Dns("localhost".into()));
        addr.push(Protocol::Tcp(port));
        addr
    }
}
