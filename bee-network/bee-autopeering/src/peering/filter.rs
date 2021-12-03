// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    peer::{peer_id::PeerId, Peer},
    NeighborValidator,
};

use std::{
    collections::HashSet,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

#[derive(Clone)]
pub(crate) struct NeighborFilter<V: NeighborValidator> {
    inner: Arc<RwLock<NeighborFilterInner<V>>>,
}

impl<V: NeighborValidator> NeighborFilter<V> {
    /// Creates a new filter.
    ///
    /// A peer id same as `local_id` will always be rejected. A `validator` must be provided
    /// to inject a user defined criteria.
    pub fn new(local_id: PeerId, validator: V) -> Self {
        Self {
            inner: Arc::new(RwLock::new(NeighborFilterInner::new(local_id, validator))),
        }
    }

    /// Adds a single peer id that should be rejected.
    pub(crate) fn add(&self, peer_id: PeerId) {
        self.write().add(peer_id);
    }

    /// Resets the filter (i.e. removes all currently rejected peer ids).
    pub(crate) fn clear(&self) {
        self.write().clear();
    }

    /// Returns `true` if the filter is okay with the candidate, otherwise `false`.
    pub(crate) fn ok(&self, candidate: impl AsRef<Peer>) -> bool {
        self.read().ok(candidate)
    }

    /// Applies the filter to a list of candidates.
    pub(crate) fn apply_list<'a, P: AsRef<Peer>>(&self, candidates: &'a [P]) -> Vec<&'a P> {
        self.read().apply_list(candidates)
    }

    fn read(&self) -> RwLockReadGuard<NeighborFilterInner<V>> {
        self.inner.read().expect("error getting read access")
    }

    fn write(&self) -> RwLockWriteGuard<NeighborFilterInner<V>> {
        self.inner.write().expect("error getting write access")
    }
}

pub(crate) struct NeighborFilterInner<V: NeighborValidator> {
    local_id: PeerId,
    rejected: HashSet<PeerId>,
    validator: V,
}

impl<V: NeighborValidator> NeighborFilterInner<V> {
    fn new(local_id: PeerId, validator: V) -> Self {
        Self {
            local_id,
            rejected: HashSet::new(),
            validator,
        }
    }

    /// Adds a single peer id that should be rejected.
    fn add(&mut self, peer_id: PeerId) {
        self.rejected.insert(peer_id);
    }

    /// Resets the filter (i.e. removes all currently rejected peer ids).
    fn clear(&mut self) {
        self.rejected.clear()
    }

    /// Returns `true` if the filter is okay with the candidate, otherwise `false`.
    fn ok(&self, candidate: impl AsRef<Peer>) -> bool {
        let peer = candidate.as_ref();
        let peer_id = peer.peer_id();

        if peer_id == &self.local_id || self.rejected.contains(peer_id) {
            false
        } else {
            self.validator.is_valid(peer)
        }
    }

    /// Applies the filter to a list of candidates.
    fn apply_list<'a, P: AsRef<Peer>>(&self, candidates: &'a [P]) -> Vec<&'a P> {
        candidates.iter().filter(|c| self.ok(*c)).collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        local::services::{ServiceProtocol, AUTOPEERING_SERVICE_NAME},
        peer::Peer,
    };

    use super::*;

    #[derive(Clone)]
    struct DummyValidator {}
    impl NeighborValidator for DummyValidator {
        fn is_valid(&self, peer: &Peer) -> bool {
            peer.services().get(AUTOPEERING_SERVICE_NAME).unwrap().port() == 1337
        }
    }

    fn setup_scenario1() -> (NeighborFilter<DummyValidator>, Peer, Peer) {
        let local_id = Peer::new_test_peer(0).into_id();
        let filter = NeighborFilter::new(local_id, DummyValidator {});

        let mut peer1 = Peer::new_test_peer(1);
        peer1.add_service(AUTOPEERING_SERVICE_NAME, ServiceProtocol::Udp, 6969);
        assert_eq!(1, peer1.num_services());

        let mut peer2 = Peer::new_test_peer(2);
        peer2.add_service(AUTOPEERING_SERVICE_NAME, ServiceProtocol::Udp, 1337);
        assert_eq!(1, peer2.num_services());

        (filter, peer1, peer2)
    }

    #[test]
    fn filter_apply() {
        let (filter, peer1, peer2) = setup_scenario1();

        assert!(!filter.read().ok(peer1));
        assert!(filter.read().ok(peer2));
    }

    #[test]
    fn filter_apply_list() {
        let (filter, peer1, peer2) = setup_scenario1();

        let candidates = [peer1, peer2];

        let included = filter.write().apply_list(&candidates);
        assert_eq!(1, included.len());
    }
}
