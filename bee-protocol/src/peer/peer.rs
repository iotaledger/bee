// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{milestone::MilestoneIndex, peer::PeerMetrics};

use bee_network::{Multiaddr, PeerId};

use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};

pub struct Peer {
    pub(crate) id: PeerId,
    pub(crate) address: Multiaddr,
    pub(crate) metrics: PeerMetrics,
    pub(crate) latest_solid_milestone_index: AtomicU32,
    pub(crate) pruned_index: AtomicU32,
    pub(crate) latest_milestone_index: AtomicU32,
    pub(crate) connected_peers: AtomicU8,
    pub(crate) synced_peers: AtomicU8,
}

impl Peer {
    pub(crate) fn new(id: PeerId, address: Multiaddr) -> Self {
        Self {
            id,
            address,
            metrics: PeerMetrics::default(),
            latest_solid_milestone_index: AtomicU32::new(0),
            pruned_index: AtomicU32::new(0),
            latest_milestone_index: AtomicU32::new(0),
            connected_peers: AtomicU8::new(0),
            synced_peers: AtomicU8::new(0),
        }
    }

    pub(crate) fn set_latest_solid_milestone_index(&self, index: MilestoneIndex) {
        self.latest_solid_milestone_index.store(*index, Ordering::Relaxed);
    }

    pub(crate) fn latest_solid_milestone_index(&self) -> MilestoneIndex {
        self.latest_solid_milestone_index.load(Ordering::Relaxed).into()
    }

    pub(crate) fn set_pruned_index(&self, index: MilestoneIndex) {
        self.pruned_index.store(*index, Ordering::Relaxed);
    }

    pub(crate) fn pruned_index(&self) -> MilestoneIndex {
        self.pruned_index.load(Ordering::Relaxed).into()
    }

    pub(crate) fn set_latest_milestone_index(&self, index: MilestoneIndex) {
        self.latest_milestone_index.store(*index, Ordering::Relaxed);
    }

    pub(crate) fn latest_milestone_index(&self) -> MilestoneIndex {
        self.latest_milestone_index.load(Ordering::Relaxed).into()
    }

    pub(crate) fn set_connected_peers(&self, connected_peers: u8) {
        self.connected_peers.store(connected_peers, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    pub(crate) fn connected_peers(&self) -> u8 {
        self.connected_peers.load(Ordering::Relaxed)
    }

    pub(crate) fn set_synced_peers(&self, synced_peers: u8) {
        self.synced_peers.store(synced_peers, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    pub(crate) fn synced_peers(&self) -> u8 {
        self.synced_peers.load(Ordering::Relaxed)
    }

    pub(crate) fn has_data(&self, index: MilestoneIndex) -> bool {
        index > self.pruned_index() && index <= self.latest_solid_milestone_index()
    }

    pub(crate) fn maybe_has_data(&self, index: MilestoneIndex) -> bool {
        index > self.pruned_index() && index <= self.latest_milestone_index()
    }
}
