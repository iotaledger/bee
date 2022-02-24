// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use super::{
    peer_data::{PeerInfo, PeerRelation},
    peer_id::PeerId,
};
use crate::{
    config::PeerConfig,
    time::{self, MINUTE},
};

use hashbrown::{HashMap, HashSet};
use libp2p::Multiaddr;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use std::sync::Arc;

#[derive(Clone, Debug, thiserror::Error)]
pub(crate) enum PeerStateMapError {
    /// A failure due to an address being added twice.
    #[error("Already added that address: {0}")]
    DuplicateAddress(Multiaddr),
    /// A failure due to attempting to connect a peer twice.
    #[error("Already connected to that peer: {0}")]
    DuplicateConnection(PeerId),
    /// A failure due to attempting to add a peer twice.
    #[error("Already added that peer: {0}")]
    DuplicatePeer(PeerId),
    /// A failure due to an address being one of the bind addresses.
    #[error("Address is one of the local addresses: {0}")]
    LocalAddress(Multiaddr),
    /// A failure due to a peer id being equal to the local id.
    #[error("Peer matches the local Id: {0}")]
    LocalPeer(PeerId),
    /// A failure due to a peer not being present in the state map.
    #[error("Missing peer: {0}")]
    MissingPeer(PeerId),
    /// A failure due to hitting the maximum number of allowed unknown peers.
    #[error("Tried to add more unknown peers than defined in the config ({0}).")]
    UnknownPeerLimit(u16),
    /// A failure due to hitting the maximum number of allowed discovered peers.
    #[error("Tried to add more discovered peers than defined in the config ({0}).")]
    DiscoveredPeerLimit(u16),
}

#[derive(Debug, Clone)]
pub(crate) struct PeerStateMapConfig {
    pub(crate) local_peer_id: PeerId,
    pub(crate) max_unknown_peers: u16,
    pub(crate) max_discovered_peers: u16,
}

/// A (thread-safe) peer state map that keeps track of each peer's connection state.
#[derive(Debug, Clone)]
pub(crate) struct PeerStateMap {
    config: PeerStateMapConfig,
    inner: Arc<RwLock<PeerStateMapInner>>,
}

#[derive(Debug)]
pub(crate) struct PeerStateMapValue {
    pub(crate) peer_info: PeerInfo,
    pub(crate) peer_state: PeerState,
    pub(crate) peer_metrics: PeerMetrics,
}

impl PeerStateMap {
    pub(crate) fn new(config: PeerStateMapConfig, manual_peers: Vec<PeerConfig>) -> Self {
        let mut peer_state_map = HashMap::default();

        peer_state_map.extend(manual_peers.into_iter().map(|peer| {
            let peer_id = peer.peer_id;
            let peer_info = PeerInfo {
                address: peer.peer_addr,
                alias: peer.peer_alias.unwrap_or_else(|| peer_id.to_string()),
                relation: PeerRelation::Known,
            };
            let peer_state = PeerState::Disconnected;
            let peer_metrics = PeerMetrics::default();

            (
                peer_id,
                PeerStateMapValue {
                    peer_info,
                    peer_state,
                    peer_metrics,
                },
            )
        }));

        Self {
            config,
            inner: Arc::new(RwLock::new(PeerStateMapInner {
                local_addrs: HashSet::default(),
                peer_state_map,
            })),
        }
    }

    pub(crate) fn config(&self) -> &PeerStateMapConfig {
        &self.config
    }

    pub(crate) fn add_local_address(&self, addr: Multiaddr) -> Result<(), PeerStateMapError> {
        self.inner_mut().add_local_addr(addr)
    }

    pub(crate) fn add_remote_peer(&self, peer_id: PeerId, peer_info: PeerInfo) -> Result<(), PeerStateMapError> {
        let mut inner = self.inner_mut(); //.add_remote_peer(peer_id, peer_info)

        // Deny adding more discovered peers than allowed.
        if peer_info.relation.is_discovered()
            && inner.count_peers_with(|v| v.peer_info.relation.is_discovered()) as u16
                >= self.config.max_discovered_peers
        {
            Err(PeerStateMapError::DiscoveredPeerLimit(self.config.max_discovered_peers))
        } else {
            inner.add_remote_peer(peer_id, peer_info)
        }
    }

    pub(crate) fn remove_peer(&self, peer_id: &PeerId) -> Result<PeerInfo, PeerStateMapError> {
        self.inner_mut().remove_peer(peer_id)
    }

    pub(crate) fn remove_peer_conditionally(
        &self,
        peer_id: &PeerId,
        cond: impl Fn(&PeerStateMapValue) -> bool,
    ) -> bool {
        self.inner_mut().remove_peer_conditionally(peer_id, cond)
    }

    pub(crate) fn contains_peer(&self, peer_id: &PeerId) -> bool {
        self.inner().contains_peer(peer_id)
    }

    #[allow(dead_code)]
    fn find_peer_by_address(&self, peer_addr: &Multiaddr) -> Option<PeerId> {
        self.inner().find_peer_by_address(peer_addr)
    }

    /// Note: Returns `false` if the peer doesn't exist.
    pub(crate) fn peer_satisfies_condition(&self, peer_id: &PeerId, cond: impl Fn(&PeerStateMapValue) -> bool) -> bool {
        self.inner().peer_satisfies_condition(peer_id, cond)
    }

    pub(crate) fn get_info(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        self.inner().get_info(peer_id)
    }

    pub(crate) fn get_info_conditionally(
        &self,
        peer_id: &PeerId,
        cond: impl Fn(&PeerStateMapValue) -> bool,
    ) -> Option<PeerInfo> {
        self.inner().get_info_conditionally(peer_id, cond)
    }

    pub(crate) fn update_peer_state(
        &self,
        peer_id: &PeerId,
        update: impl FnMut(&mut PeerState),
    ) -> Result<(), PeerStateMapError> {
        self.inner_mut().update_peer_state(peer_id, update)
    }

    pub(crate) fn update_last_ping(&self, peer_id: PeerId) {
        self.inner_mut().update_last_ping(peer_id);
    }

    pub(crate) fn update_last_identify(&self, peer_id: PeerId) {
        self.inner_mut().update_last_identify(peer_id);
    }

    pub(crate) fn count_peers_with(&self, cond: impl Fn(&PeerStateMapValue) -> bool) -> usize {
        self.inner().count_peers_with(cond)
    }

    pub(crate) fn gen_stats(&self) -> PeerStateMapStats {
        let inner = self.inner();

        PeerStateMapStats {
            num_all: inner.len() as u16,
            num_connected: inner.count_peers_with(|v| v.peer_state.is_connected()) as u16,
            num_disconnected: inner.count_peers_with(|v| v.peer_state.is_disconnected()) as u16,
            num_known: inner.count_peers_with(|v| v.peer_info.relation.is_known()) as u16,
            num_known_and_connected: inner
                .count_peers_with(|v| v.peer_info.relation.is_known() && v.peer_state.is_connected())
                as u16,
            num_unknown: inner.count_peers_with(|v| v.peer_info.relation.is_unknown()) as u16,
            num_unknown_and_connected: inner
                .count_peers_with(|v| v.peer_info.relation.is_unknown() && v.peer_state.is_connected())
                as u16,
            max_unknown: self.config().max_unknown_peers,
            num_discovered: inner.count_peers_with(|v| v.peer_info.relation.is_discovered()) as u16,
            num_discovered_and_connected: inner
                .count_peers_with(|v| v.peer_info.relation.is_discovered() && v.peer_state.is_connected())
                as u16,
            max_discovered: self.config().max_discovered_peers,
            // FIXME: unwrap
            num_identify_expired: inner
                .count_peers_with(|v| time::since(v.peer_metrics.last_identify).unwrap() > 5 * MINUTE)
                as u16,
            num_ping_expired: inner.count_peers_with(|v| time::since(v.peer_metrics.last_ping).unwrap() > 10 * MINUTE)
                as u16,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.inner().len()
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn clear(&self) {
        let mut inner = self.inner_mut();
        inner.local_addrs.clear();
        inner.peer_state_map.clear();
        drop(inner);
    }

    pub(crate) fn accept_inbound_peer(&self, peer_id: &PeerId, peer_addr: &Multiaddr) -> InboundPeerAcceptance {
        use InboundPeerAcceptance::*;

        // Deny if inbound peer uses the same peer id.
        if peer_id == &self.config.local_peer_id {
            log::trace!("Denied inbound peer: {}", PeerStateMapError::LocalPeer(*peer_id));
            return Deny;
        }

        // Deny if inbound peer uses any of our local addresses.
        if self.inner().local_addrs.contains(peer_addr) {
            log::trace!(
                "Denied inbound peer: {}",
                PeerStateMapError::LocalAddress(peer_addr.clone())
            );
            return Deny;
        }

        // Deny if inbound peer is already connected.
        if self.peer_satisfies_condition(peer_id, |v| v.peer_state.is_connected()) {
            log::trace!(
                "Denied inbound peer: {}",
                PeerStateMapError::DuplicateConnection(*peer_id)
            );
            return Deny;
        }

        if !self.contains_peer(peer_id) {
            // ~> not contained, hence not connected
            if self.count_peers_with(|v| v.peer_info.relation.is_unknown()) as u16 >= self.config.max_unknown_peers {
                // Deny if inbound peer is unknown but all slots for unknown peers are already occupied.
                log::trace!(
                    "Denied inbound peer: {}",
                    PeerStateMapError::UnknownPeerLimit(self.config.max_unknown_peers)
                );
                Deny
            } else {
                // Accept inbound peer as unknown.
                log::trace!("Accepted unknown inbound peer {peer_id}.");
                AcceptUnknown
            }
        } else {
            // ~> contained but not connected => always accept
            // Accept inbound peer as known or discovered.
            log::trace!("Accepted known or discovered inbound peer {peer_id}.");
            AcceptKnownOrDiscovered
        }
    }

    pub(crate) fn accept_outbound_peer(&self, peer_id: &PeerId) -> bool {
        if peer_id == &self.config.local_peer_id {
            // Deny if outbound peer uses the same peer id.
            log::trace!("Denied outbound peer: {}", PeerStateMapError::LocalPeer(*peer_id));
            false
        } else if !self.contains_peer(peer_id) {
            // Deny if outbound peer was not added beforehand.
            log::trace!("Denied outbound peer: {}", PeerStateMapError::MissingPeer(*peer_id));
            false
        } else if self.peer_satisfies_condition(peer_id, |v| v.peer_state.is_connected()) {
            // Deny if outbound peer is already connected.
            log::trace!(
                "Denied outbound peer: {}",
                PeerStateMapError::DuplicateConnection(*peer_id)
            );
            false
        } else {
            true
        }
    }

    fn inner(&self) -> RwLockReadGuard<'_, PeerStateMapInner> {
        self.inner.read()
    }

    fn inner_mut(&self) -> RwLockWriteGuard<'_, PeerStateMapInner> {
        self.inner.write()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum InboundPeerAcceptance {
    Deny,
    AcceptUnknown,
    AcceptKnownOrDiscovered,
}

impl InboundPeerAcceptance {
    pub(crate) fn is_accepted(&self) -> bool {
        matches!(self, Self::AcceptUnknown | Self::AcceptKnownOrDiscovered)
    }
}

#[derive(Debug)]
pub(crate) struct PeerStateMapInner {
    local_addrs: HashSet<Multiaddr>,
    peer_state_map: HashMap<PeerId, PeerStateMapValue>,
}

impl PeerStateMapInner {
    fn add_local_addr(&mut self, addr: Multiaddr) -> Result<(), PeerStateMapError> {
        if self.local_addrs.contains(&addr) {
            Err(PeerStateMapError::DuplicateAddress(addr))
        } else {
            self.local_addrs.insert(addr);
            Ok(())
        }
    }

    fn add_remote_peer(&mut self, peer_id: PeerId, peer_info: PeerInfo) -> Result<(), PeerStateMapError> {
        if self.contains_peer(&peer_id) {
            // Deny adding the same peer twice.
            Err(PeerStateMapError::DuplicatePeer(peer_id))
        } else {
            self.peer_state_map.insert(
                peer_id,
                PeerStateMapValue {
                    peer_info,
                    peer_state: PeerState::Disconnected,
                    peer_metrics: PeerMetrics::default(),
                },
            );
            Ok(())
        }
    }

    fn remove_peer(&mut self, peer_id: &PeerId) -> Result<PeerInfo, PeerStateMapError> {
        let v = self
            .peer_state_map
            .remove(peer_id)
            .ok_or(PeerStateMapError::MissingPeer(*peer_id))?;

        Ok(v.peer_info)
    }

    fn remove_peer_conditionally(&mut self, peer_id: &PeerId, cond: impl Fn(&PeerStateMapValue) -> bool) -> bool {
        if self.peer_state_map.get(peer_id).filter(|v| cond(*v)).is_some() {
            // Panic:
            // We checked above that the element exists.
            self.peer_state_map.remove(peer_id).expect("remove peer");
            true
        } else {
            false
        }
    }

    fn contains_peer(&self, peer_id: &PeerId) -> bool {
        self.peer_state_map.contains_key(peer_id)
    }

    fn find_peer_by_address(&self, peer_addr: &Multiaddr) -> Option<PeerId> {
        self.peer_state_map
            .iter()
            .find(|(_, v)| v.peer_info.address == *peer_addr)
            .map(|(p, _)| *p)
    }

    fn peer_satisfies_condition(&self, peer_id: &PeerId, cond: impl Fn(&PeerStateMapValue) -> bool) -> bool {
        self.peer_state_map.get(peer_id).map_or(false, cond)
    }

    fn get_info(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        self.peer_state_map.get(peer_id).map(|v| v.peer_info.clone())
    }

    fn get_info_conditionally(&self, peer_id: &PeerId, cond: impl Fn(&PeerStateMapValue) -> bool) -> Option<PeerInfo> {
        self.peer_state_map
            .get(peer_id)
            .filter(|v| cond(*v))
            .map(|v| v.peer_info.clone())
    }

    fn update_peer_state(
        &mut self,
        peer_id: &PeerId,
        mut update: impl FnMut(&mut PeerState),
    ) -> Result<(), PeerStateMapError> {
        let v = self
            .peer_state_map
            .get_mut(peer_id)
            .ok_or(PeerStateMapError::MissingPeer(*peer_id))?;

        update(&mut v.peer_state);

        Ok(())
    }

    fn update_last_ping(&mut self, peer_id: PeerId) {
        if let Some(v) = self.peer_state_map.get_mut(&peer_id) {
            v.peer_metrics.last_ping = crate::time::unix_now_secs();
        }
    }

    fn update_last_identify(&mut self, peer_id: PeerId) {
        if let Some(v) = self.peer_state_map.get_mut(&peer_id) {
            v.peer_metrics.last_identify = crate::time::unix_now_secs();
        }
    }

    fn count_peers_with(&self, cond: impl Fn(&PeerStateMapValue) -> bool) -> usize {
        self.peer_state_map
            .iter()
            .map(|(_, v)| if cond(v) { 1 } else { 0 })
            .sum()
    }

    fn len(&self) -> usize {
        self.peer_state_map.len()
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn clear(&mut self) {
        self.peer_state_map.clear();
    }
}

#[derive(Clone, Debug)]
pub(crate) enum PeerState {
    Disconnected,
    Connected,
}

impl PeerState {
    pub(crate) fn is_disconnected(&self) -> bool {
        matches!(self, Self::Disconnected)
    }

    pub(crate) fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }

    pub(crate) fn set_connected(&mut self) {
        *self = Self::Connected
    }

    pub(crate) fn set_disconnected(&mut self) {
        *self = Self::Disconnected
    }
}

impl Default for PeerState {
    fn default() -> Self {
        Self::Disconnected
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
#[allow(dead_code)]
pub(crate) struct PeerStateMapStats {
    pub(crate) num_all: u16,
    pub(crate) num_known: u16,
    pub(crate) num_known_and_connected: u16,
    pub(crate) num_unknown: u16,
    pub(crate) max_unknown: u16,
    pub(crate) num_unknown_and_connected: u16,
    pub(crate) num_discovered: u16,
    pub(crate) num_discovered_and_connected: u16,
    pub(crate) max_discovered: u16,
    pub(crate) num_connected: u16,
    pub(crate) num_disconnected: u16,
    pub(crate) num_ping_expired: u16,
    pub(crate) num_identify_expired: u16,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PeerMetrics {
    last_ping: u64,
    last_identify: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::{identity::ed25519::Keypair, multiaddr::Protocol};

    #[test]
    fn empty_peer_state_map() {
        let local_peer_id = gen_constant_peer_id();
        let config = PeerStateMapConfig {
            local_peer_id,
            max_unknown_peers: 4,
            max_discovered_peers: 8,
        };

        let psm = PeerStateMap::new(config, vec![]);

        assert_eq!(psm.len(), 0);
    }

    #[test]
    fn add_remote_peers() {
        let local_peer_id = gen_constant_peer_id();
        let config = PeerStateMapConfig {
            local_peer_id,
            max_unknown_peers: 4,
            max_discovered_peers: 8,
        };

        let psm = PeerStateMap::new(config, vec![]);

        for i in 1..=3 {
            assert!(
                psm.add_remote_peer(
                    gen_random_peer_id(),
                    gen_deterministic_peer_info(i, PeerRelation::Known)
                )
                .is_ok()
            );
            assert_eq!(psm.len(), i as usize);
        }
    }

    #[test]
    fn insert() {
        let local_peer_id = gen_constant_peer_id();
        let config = PeerStateMapConfig {
            local_peer_id,
            max_unknown_peers: 4,
            max_discovered_peers: 8,
        };

        let psm = PeerStateMap::new(config, vec![]);

        let remote_peer_id = gen_constant_peer_id();

        assert!(psm.add_remote_peer(remote_peer_id, gen_constant_peer_info()).is_ok());

        // Do not allow inserting the same peer id twice.
        assert!(matches!(
            psm.add_remote_peer(remote_peer_id, gen_constant_peer_info()),
            Err(PeerStateMapError::DuplicatePeer(_))
        ));
    }

    #[test]
    fn conditional_remove() {
        let local_peer_id = gen_constant_peer_id();
        let config = PeerStateMapConfig {
            local_peer_id,
            max_unknown_peers: 4,
            max_discovered_peers: 8,
        };
        let psm = PeerStateMap::new(config, vec![]);

        let peer_id = gen_random_peer_id();

        psm.add_remote_peer(peer_id, gen_deterministic_peer_info(0, PeerRelation::Known))
            .unwrap();
        assert_eq!(1, psm.len());

        psm.remove_peer_conditionally(&peer_id, |v| v.peer_info.relation.is_unknown());
        assert_eq!(1, psm.len());

        psm.remove_peer_conditionally(&peer_id, |v| v.peer_info.relation.is_known());
        assert_eq!(0, psm.len());
    }

    #[test]
    fn default_peer_state() {
        let peerstate = PeerState::default();

        assert!(peerstate.is_disconnected());
    }

    pub(crate) fn gen_constant_peer_id() -> PeerId {
        "12D3KooWJWEKvSFbben74C7H4YtKjhPMTDxd7gP7zxWSUEeF27st"
            .parse::<libp2p_core::PeerId>()
            .unwrap()
            .into()
    }

    pub(crate) fn gen_random_peer_id() -> PeerId {
        libp2p_core::PeerId::from_public_key(&libp2p_core::PublicKey::Ed25519(Keypair::generate().public())).into()
    }

    pub(crate) fn gen_deterministic_peer_info(port: u16, relation: PeerRelation) -> PeerInfo {
        PeerInfo {
            address: gen_deterministic_addr(port),
            alias: port.to_string(),
            relation,
        }
    }

    pub(crate) fn gen_constant_peer_info() -> PeerInfo {
        PeerInfo {
            address: gen_deterministic_addr(1),
            alias: String::new(),
            relation: PeerRelation::Known,
        }
    }

    pub(crate) fn gen_deterministic_addr(port: u16) -> Multiaddr {
        let mut addr = Multiaddr::empty();
        addr.push(Protocol::Dns("localhost".into()));
        addr.push(Protocol::Tcp(port));
        addr
    }
}
