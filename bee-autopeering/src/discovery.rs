// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{identity::PeerId, peer::Peer};

use std::{collections::VecDeque, fmt};

// From `iotaledger/hive.go`:
// time interval after which the next peer is reverified
const DEFAULT_REVERIFY_INTERVAL_SECS: u64 = 10;
// time interval after which peers are queried for new peers
const DEFAULT_QUERY_INTERVAL_SECS: u64 = 60;
// maximum number of peers that can be managed
const DEFAULT_MAX_MANAGED: usize = 1000;
// maximum number of peers kept in the replacement list
const DEFAULT_MAX_REPLACEMENTS: usize = 10;

#[derive(Debug)]
pub(crate) enum Event {
    PeerDiscovered { peer: Peer },
    PeerDeleted { peer_id: PeerId },
}

pub(crate) struct DiscoveredPeer {
    peer: Peer,
    // how often that peer has been re-verified
    verified_count: usize,
    // number of returned new peers when queried the last time
    last_new_peers: usize,
}

impl DiscoveredPeer {
    pub fn peer(&self) -> &Peer {
        &self.peer
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer.peer_id()
    }

    pub fn verified_count(&self) -> usize {
        self.verified_count
    }

    pub fn last_new_peers(&self) -> usize {
        self.last_new_peers
    }

    pub fn into_peer(self) -> Peer {
        self.peer
    }
}

impl From<Peer> for DiscoveredPeer {
    fn from(peer: Peer) -> Self {
        Self {
            peer,
            verified_count: 0,
            last_new_peers: 0,
        }
    }
}

impl fmt::Debug for DiscoveredPeer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiscoveredPeer")
            .field("peer", &self.peer)
            .field("verified_count", &self.verified_count)
            .field("last_new_peers", &self.last_new_peers)
            .finish()
    }
}

pub(crate) struct DiscoveredPeerlist {
    peers: VecDeque<DiscoveredPeer>,
}

impl DiscoveredPeerlist {
    pub fn new() -> Self {
        Self {
            peers: VecDeque::with_capacity(DEFAULT_MAX_MANAGED),
        }
    }

    pub fn remove(&mut self, peer_id: PeerId) {
        if let Some(index) = self.peers.iter().position(|peer| peer.peer_id() == peer_id) {
            self.remove_at(index)
        }
    }

    pub fn remove_at(&mut self, index: usize) {
        self.peers.remove(index);
    }

    pub fn push_front(&mut self, peer: DiscoveredPeer) {
        self.peers.push_front(peer)
    }

    pub fn push_back(&mut self, peer: DiscoveredPeer) {
        self.peers.push_back(peer)
    }

    pub fn pop_front(&mut self) {
        self.peers.pop_front();
    }

    pub fn pop_back(&mut self) {
        self.peers.pop_back();
    }
}

pub(crate) struct DiscoveryManager {}

impl DiscoveryManager {
    pub(crate) fn new() -> Self {
        Self {}
    }
    pub(crate) async fn run(self) {
        todo!()
    }
}
