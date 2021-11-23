// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use super::{
    error::Error,
    info::{PeerInfo, PeerRelation},
};

use crate::{alias, config::Peer, init::global, swarm::protocols::iota_gossip::GossipSender};

use hashbrown::{HashMap, HashSet};
use libp2p::{Multiaddr, PeerId};
use tokio::sync::RwLock;

use std::{mem::take, sync::Arc};

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
            // Realiasing because of otherwise partial move.
            let peer_id = peer.peer_id;
            (
                peer_id,
                (
                    PeerInfo {
                        address: peer.multiaddr,
                        alias: peer.alias.unwrap_or_else(|| alias!(peer_id).to_owned()),
                        relation: PeerRelation::Known,
                    },
                    PeerState::Disconnected,
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
            return Err((peer_id, peer_info, Error::PeerIsDuplicate(peer_id)));
        }

        // Since we already checked that such a `peer_id` is not yet present, the returned value is always `None`.
        let _ = self.peers.insert(peer_id, (peer_info, PeerState::Disconnected));

        Ok(())
    }

    pub fn insert_local_addr(&mut self, addr: Multiaddr) -> Result<(), (Multiaddr, Error)> {
        if self.local_addrs.contains(&addr) {
            return Err((addr.clone(), Error::AddressIsDuplicate(addr)));
        }

        let _ = self.local_addrs.insert(addr);

        Ok(())
    }

    pub fn remove(&mut self, peer_id: &PeerId) -> Result<PeerInfo, Error> {
        let (info, _) = self.peers.remove(peer_id).ok_or(Error::PeerNotPresent(*peer_id))?;

        Ok(info)
    }

    pub fn contains(&self, peer_id: &PeerId) -> bool {
        self.peers.contains_key(peer_id)
    }

    pub fn info(&self, peer_id: &PeerId) -> Result<PeerInfo, Error> {
        self.peers
            .get(peer_id)
            .ok_or(Error::PeerNotPresent(*peer_id))
            .map(|(peer_info, _)| peer_info.clone())
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }

    pub fn update_info<U>(&mut self, peer_id: &PeerId, mut update: U) -> Result<(), Error>
    where
        U: FnMut(&mut PeerInfo),
    {
        let (info, _) = self.peers.get_mut(peer_id).ok_or(Error::PeerNotPresent(*peer_id))?;

        update(info);

        Ok(())
    }

    pub fn update_state<U>(&mut self, peer_id: &PeerId, mut update: U) -> Result<Option<GossipSender>, Error>
    where
        U: FnMut(&mut PeerState) -> Option<GossipSender>,
    {
        let (_, state) = self.peers.get_mut(peer_id).ok_or(Error::PeerNotPresent(*peer_id))?;

        Ok(update(state))
    }

    pub fn satisfies<P>(&self, peer_id: &PeerId, predicate: P) -> Result<bool, Error>
    where
        P: Fn(&PeerInfo, &PeerState) -> bool,
    {
        self.peers
            .get(peer_id)
            .ok_or(Error::PeerNotPresent(*peer_id))
            .map(|(info, state)| predicate(info, state))
    }

    pub fn filter_info<'a, P: 'a>(&'a self, predicate: P) -> impl Iterator<Item = (PeerId, PeerInfo)> + '_
    where
        P: Fn(&PeerInfo, &PeerState) -> bool,
    {
        self.peers.iter().filter_map(move |(peer_id, (info, state))| {
            if predicate(info, state) {
                Some((*peer_id, info.clone()))
            } else {
                None
            }
        })
    }

    pub fn filter_count<P>(&self, predicate: P) -> usize
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

    pub fn filter_remove<P>(&mut self, peer_id: &PeerId, predicate: P) -> bool
    where
        P: Fn(&PeerInfo, &PeerState) -> bool,
    {
        // NB: Since we drop a potential reference to `&(PeerInfo, PeerState)` this code won't create a deadlock in case
        // we refactor `PeerList` in a way that `.remove` would only take a `&self`.

        if self
            .peers
            .get(peer_id)
            .filter(|(info, state)| predicate(info, state))
            .is_some()
        {
            // Should always return `true`, because we know it's there.
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

    pub fn unban_address(&mut self, addr: &Multiaddr) -> Result<(), Error> {
        if self.banned_addrs.remove(addr) {
            Ok(())
        } else {
            Err(Error::AddressIsUnbanned(addr.clone()))
        }
    }

    pub fn is_peer_banned(&self, peer_id: &PeerId) -> bool {
        self.banned_peers.contains(peer_id)
    }

    pub fn is_addr_banned(&self, addr: &Multiaddr) -> bool {
        self.banned_addrs.contains(addr)
    }

    pub fn accepts_incoming_peer(&self, peer_id: &PeerId, peer_addr: &Multiaddr) -> Result<(), Error> {
        // Checks performed are:
        // - Deny ourself as peer.
        // - Deny one of our own addresses.
        // - Deny banned peers.
        // - Deny banned addresses.
        // - Deny already connected peers.
        // - Deny more than the configured unknown peers.
        if peer_id == &self.local_id {
            Err(Error::PeerIsLocal(*peer_id))
        } else if self.local_addrs.contains(peer_addr) {
            Err(Error::AddressIsLocal(peer_addr.clone()))
        } else if self.banned_peers.contains(peer_id) {
            Err(Error::PeerIsBanned(*peer_id))
        } else if self.banned_addrs.contains(peer_addr) {
            Err(Error::AddressIsBanned(peer_addr.clone()))
        } else if self
            .satisfies(peer_id, |_, state| state.is_connected())
            .unwrap_or(false)
        {
            Err(Error::PeerIsConnected(*peer_id))
        } else if !self.contains(peer_id)
            && self.filter_count(|info, _| info.relation.is_unknown()) >= global::max_unknown_peers()
        {
            Err(Error::ExceedsUnknownPeerLimit(global::max_unknown_peers()))
        } else if !self.contains(peer_id)
            && self.filter_count(|info, _| info.relation.is_discovered()) >= global::max_discovered_peers()
        {
            Err(Error::ExceedsDiscoveredPeerLimit(global::max_discovered_peers()))
        } else {
            // All checks passed! Accept that peer.
            Ok(())
        }
    }

    pub fn allows_dialing_peer(&self, peer_id: &PeerId) -> Result<(), Error> {
        // Checks performed are:
        // - Deny dialing ourself as peer.
        // - Deny dialing a peer that has not been added first. TODO: check if we might want to allow this!
        // - Deny dialing a banned peer.
        // - Deny dialing an already connected peer.
        // - Deny dialing a local address.
        // - Deny dialing a banned address.
        // - Deny dialing more than configured unkown peers.
        if peer_id == &self.local_id {
            Err(Error::PeerIsLocal(*peer_id))
        } else if !self.contains(peer_id) {
            Err(Error::PeerNotPresent(*peer_id))
        } else if self.banned_peers.contains(peer_id) {
            Err(Error::PeerIsBanned(*peer_id))
        } else if self
            .satisfies(peer_id, |_, state| state.is_connected())
            .unwrap_or(false)
        {
            Err(Error::PeerIsConnected(*peer_id))
        } else {
            let (peer_info, _) = self.peers.get(peer_id).unwrap();

            if self.local_addrs.contains(&peer_info.address) {
                Err(Error::AddressIsLocal(peer_info.address.clone()))
            } else if self.banned_addrs.contains(&peer_info.address) {
                Err(Error::AddressIsBanned(peer_info.address.clone()))
            } else if peer_info.relation.is_unknown()
                && self.filter_count(|info, _| info.relation.is_unknown()) >= global::max_unknown_peers()
            {
                Err(Error::ExceedsUnknownPeerLimit(global::max_unknown_peers()))
            } else if peer_info.relation.is_discovered()
                && self.filter_count(|info, _| info.relation.is_discovered()) >= global::max_discovered_peers()
            {
                Err(Error::ExceedsDiscoveredPeerLimit(global::max_discovered_peers()))
            } else {
                // All checks passed! Allow dialing that peer.
                Ok(())
            }
        }
    }

    pub fn allows_dialing_addr(&self, addr: &Multiaddr) -> Result<(), Error> {
        // Checks performed are:
        // - Deny dialing a local address.
        // - Deny dialing a banned address.
        // - Deny dialing an already connected peer (with that address).
        if self.local_addrs.contains(addr) {
            Err(Error::AddressIsLocal(addr.clone()))
        } else if self.banned_addrs.contains(addr) {
            Err(Error::AddressIsBanned(addr.clone()))
        } else if let Some(peer_id) = self.find_peer_if_connected(addr) {
            Err(Error::PeerIsConnected(peer_id))
        } else {
            // All checks passed! Allow dialing that address.
            Ok(())
        }
    }

    fn find_peer_if_connected(&self, addr: &Multiaddr) -> Option<PeerId> {
        self.filter_info(|info, state| state.is_connected() && info.address == *addr)
            .next()
            .map(|(peer_id, _)| peer_id)
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
    fn insert() {
        let local_id = gen_constant_peer_id();
        let mut pl = PeerList::new(local_id);

        let peer_id = gen_constant_peer_id();

        assert!(pl.insert_peer(peer_id, gen_constant_peer_info()).is_ok());

        // Do not allow inserting the same peer id twice.
        assert!(matches!(
            pl.insert_peer(peer_id, gen_constant_peer_info()),
            Err((_, _, Error::PeerIsDuplicate(_)))
        ));
    }

    #[test]
    fn deny_incoming_local_peer() {
        let local_id = gen_constant_peer_id();

        let pl = PeerList::new(local_id);

        assert!(matches!(
            pl.accepts_incoming_peer(&local_id, &gen_constant_peer_info().address),
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
        pl.accepts_incoming_peer(&peer_id, &peer_info.address).unwrap();
    }

    #[test]
    fn conditional_remove() {
        let local_id = gen_constant_peer_id();
        let mut pl = PeerList::new(local_id);

        let peer_id = gen_random_peer_id();

        pl.insert_peer(peer_id, gen_deterministic_peer_info(0, PeerRelation::Known))
            .unwrap();
        assert_eq!(1, pl.len());

        pl.filter_remove(&peer_id, |info, _| info.relation.is_unknown());
        assert_eq!(1, pl.len());

        pl.filter_remove(&peer_id, |info, _| info.relation.is_known());
        assert_eq!(0, pl.len());
    }

    // ===== helpers =====

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

#[derive(Clone, Debug)]
pub enum PeerState {
    Disconnected,
    Connected(GossipSender),
}

impl Default for PeerState {
    fn default() -> Self {
        Self::Disconnected
    }
}

impl PeerState {
    pub fn is_disconnected(&self) -> bool {
        matches!(self, Self::Disconnected)
    }

    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected(_))
    }

    pub fn set_connected(&mut self, gossip_sender: GossipSender) -> Option<GossipSender> {
        *self = Self::Connected(gossip_sender);
        None
    }

    pub fn set_disconnected(&mut self) -> Option<GossipSender> {
        match take(self) {
            Self::Disconnected => None,
            Self::Connected(sender) => Some(sender),
        }
    }
}

#[cfg(test)]
mod peerstate_tests {
    use super::*;
    use crate::swarm::protocols::iota_gossip::channel;

    #[test]
    fn new_peer_state() {
        let peerstate = PeerState::default();

        assert!(peerstate.is_disconnected());
    }

    #[test]
    fn peer_state_change() {
        let mut peerstate = PeerState::Disconnected;
        let (tx, _rx) = channel();

        peerstate.set_connected(tx);
        assert!(peerstate.is_connected());

        assert!(peerstate.set_disconnected().is_some());
        assert!(peerstate.is_disconnected());
        assert!(peerstate.set_disconnected().is_none());
    }
}
