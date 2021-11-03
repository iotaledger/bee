// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::peer::{peer_id::PeerId, peerlist::ActivePeer, Peer};

use std::{
    collections::{hash_set, HashSet},
    iter,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub(crate) type Matcher = Box<dyn Fn(&Peer) -> bool + Send + Sync>;

#[derive(Clone, Default)]
pub(crate) struct ExclusionFilter {
    inner: Arc<RwLock<ExclusionFilterInner>>,
}

impl ExclusionFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&self) -> RwLockReadGuard<ExclusionFilterInner> {
        self.inner.read().expect("error getting read access")
    }

    pub fn write(&self) -> RwLockWriteGuard<ExclusionFilterInner> {
        self.inner.write().expect("error getting write access")
    }
}

#[derive(Default)]
pub(crate) struct ExclusionFilterInner {
    excluded: HashSet<PeerId>,
    matchers: Vec<Matcher>,
}

impl ExclusionFilterInner {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn exclude_peer(&mut self, peer_id: PeerId) {
        self.excluded.insert(peer_id);
    }

    pub(crate) fn exclude_peers(&mut self, peers: impl Iterator<Item = PeerId>) {
        self.excluded.extend(peers)
    }

    pub(crate) fn remove_excluded(&mut self, peer_id: &PeerId) {
        let _ = self.excluded.remove(peer_id);
    }

    pub(crate) fn clear_excluded(&mut self) {
        self.excluded.clear()
    }

    pub(crate) fn add_matcher(&mut self, matcher: Matcher) {
        self.matchers.push(matcher);
    }

    pub(crate) fn clear_matchers(&mut self) {
        self.matchers.clear()
    }

    /// Returns `true` if the filter is okay with the candidate, otherwise `false`.
    pub(crate) fn ok(&self, candidate: impl AsRef<Peer>) -> bool {
        if !self.excluded.contains(candidate.as_ref().peer_id()) {
            // All matchers must be satisfied.
            for matcher in &self.matchers {
                if !matcher(candidate.as_ref()) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Applies the filter to a list of candidates returning a subset that passed the filter.
    pub(crate) fn apply_list<'a, P: AsRef<Peer>>(&self, candidates: &'a [P]) -> Vec<&'a P> {
        let mut included = Vec::with_capacity(candidates.len());

        'candidate: for candidate in candidates {
            let peer = candidate.as_ref();

            if self.excluded.contains(peer.peer_id()) {
                continue 'candidate;
            }

            for matcher in &self.matchers {
                if !matcher(peer) {
                    continue 'candidate;
                }
            }

            included.push(candidate);
        }

        included
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &PeerId> {
        self.excluded.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::local::service_map::{ServiceTransport, AUTOPEERING_SERVICE_NAME};

    use super::*;

    impl ExclusionFilterInner {
        pub(crate) fn num_matchers(&self) -> usize {
            self.matchers.len()
        }
    }

    fn setup_scenario1() -> (ExclusionFilter, Peer, Peer) {
        let filter = ExclusionFilter::new();
        assert_eq!(0, filter.read().num_matchers());

        let matcher = |peer: &Peer| -> bool { peer.services().get(AUTOPEERING_SERVICE_NAME).unwrap().port() == 1337 };
        let matcher = Box::new(matcher);

        filter.write().add_matcher(matcher);
        assert_eq!(1, filter.read().num_matchers());

        let mut peer1 = Peer::new_test_peer(1);
        peer1.add_service(AUTOPEERING_SERVICE_NAME, ServiceTransport::Udp, 6969);
        assert_eq!(1, peer1.num_services());

        let mut peer2 = Peer::new_test_peer(2);
        peer2.add_service(AUTOPEERING_SERVICE_NAME, ServiceTransport::Udp, 1337);
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

        filter.write().clear_matchers();
        assert_eq!(0, filter.read().num_matchers());

        let included = filter.write().apply_list(&candidates);
        assert_eq!(2, included.len());
    }
}
