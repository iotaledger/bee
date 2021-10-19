// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, iter};

use crate::{distance::PeerDistance, Peer, PeerId};

type Condition = Box<dyn Fn(&Peer) -> bool>;

#[derive(Default)]
pub(crate) struct Filter {
    peers: HashMap<PeerId, bool>,
    conditions: Vec<Condition>,
}

impl Filter {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    pub(crate) fn add_peers(&mut self, peers: &[Peer]) {
        self.peers
            .extend(peers.iter().map(|p| p.peer_id()).zip(iter::repeat(true)));
    }
    pub(crate) fn remove_peer(&mut self, peer_id: &PeerId) {
        let _ = self.peers.remove(peer_id);
    }
    pub(crate) fn include_peer(&mut self, peer_id: PeerId) {
        self.peers.insert(peer_id, true);
    }
    pub(crate) fn exclude_peer(&mut self, peer_id: PeerId) {
        self.peers.insert(peer_id, false);
    }
    pub(crate) fn clear_peers(&mut self) {
        self.peers.clear()
    }
    pub(crate) fn add_condition(&mut self, condition: Condition) {
        self.conditions.push(condition);
    }
    pub(crate) fn clear_conditions(&mut self) {
        self.conditions.clear()
    }
    pub(crate) fn apply<'a, P: AsRef<Peer>>(&self, candidates: &'a [P]) -> Vec<&'a P> {
        let mut filtered = Vec::with_capacity(candidates.len());

        'candidate: for candidate in candidates {
            let peer = candidate.as_ref();

            if *self.peers.get(&peer.peer_id()).unwrap_or(&false) {
                continue 'candidate;
            }

            for condition in &self.conditions {
                if !condition(peer) {
                    continue 'candidate;
                }
            }

            filtered.push(candidate);
        }

        filtered
    }
}

#[cfg(test)]
mod tests {
    use crate::service_map::AUTOPEERING_SERVICE_NAME;

    use super::*;

    impl Filter {
        pub(crate) fn num_conditions(&self) -> usize {
            self.conditions.len()
        }
    }

    #[test]
    fn add_condition() {
        let mut filter = Filter::new();
        assert_eq!(0, filter.num_conditions());

        let condition = |peer: &Peer| -> bool { peer.services().port(AUTOPEERING_SERVICE_NAME).unwrap() == 8080 };
        let condition = Box::new(condition);

        filter.add_condition(condition);
        assert_eq!(1, filter.num_conditions());

        filter.clear_conditions();
        assert_eq!(0, filter.num_conditions());
    }

    #[test]
    fn add_peers_to_filter() {}
}
