// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::peer::PeerMetrics;

use bee_message::milestone::MilestoneIndex;
use bee_network::{Multiaddr, PeerId, PeerInfo, PeerRelation};

use std::{
    sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Peer {
    id: PeerId,
    info: PeerInfo,
    metrics: PeerMetrics,
    solid_milestone_index: AtomicU32,
    pruned_index: AtomicU32,
    latest_milestone_index: AtomicU32,
    connected_peers: AtomicU8,
    synced_peers: AtomicU8,
    heartbeat_sent_timestamp: AtomicU64,
    heartbeat_received_timestamp: AtomicU64,
}

impl Peer {
    pub(crate) fn new(id: PeerId, info: PeerInfo) -> Self {
        Self {
            id,
            info,
            metrics: PeerMetrics::default(),
            solid_milestone_index: AtomicU32::new(0),
            pruned_index: AtomicU32::new(0),
            latest_milestone_index: AtomicU32::new(0),
            connected_peers: AtomicU8::new(0),
            synced_peers: AtomicU8::new(0),
            heartbeat_sent_timestamp: AtomicU64::new(0),
            heartbeat_received_timestamp: AtomicU64::new(0),
        }
    }

    pub fn id(&self) -> &PeerId {
        &self.id
    }

    pub fn address(&self) -> &Multiaddr {
        &self.info.address
    }

    pub fn alias(&self) -> &String {
        &self.info.alias
    }

    pub fn relation(&self) -> PeerRelation {
        self.info.relation
    }

    pub fn metrics(&self) -> &PeerMetrics {
        &self.metrics
    }

    pub(crate) fn set_solid_milestone_index(&self, index: MilestoneIndex) {
        self.solid_milestone_index.store(*index, Ordering::Relaxed);
    }

    pub fn solid_milestone_index(&self) -> MilestoneIndex {
        self.solid_milestone_index.load(Ordering::Relaxed).into()
    }

    pub(crate) fn set_pruned_index(&self, index: MilestoneIndex) {
        self.pruned_index.store(*index, Ordering::Relaxed);
    }

    pub fn pruned_index(&self) -> MilestoneIndex {
        self.pruned_index.load(Ordering::Relaxed).into()
    }

    pub(crate) fn set_latest_milestone_index(&self, index: MilestoneIndex) {
        self.latest_milestone_index.store(*index, Ordering::Relaxed);
    }

    pub fn latest_milestone_index(&self) -> MilestoneIndex {
        self.latest_milestone_index.load(Ordering::Relaxed).into()
    }

    pub(crate) fn set_connected_peers(&self, connected_peers: u8) {
        self.connected_peers.store(connected_peers, Ordering::Relaxed);
    }

    pub fn connected_peers(&self) -> u8 {
        self.connected_peers.load(Ordering::Relaxed)
    }

    pub(crate) fn set_synced_peers(&self, synced_peers: u8) {
        self.synced_peers.store(synced_peers, Ordering::Relaxed);
    }

    pub fn synced_peers(&self) -> u8 {
        self.synced_peers.load(Ordering::Relaxed)
    }

    pub(crate) fn set_heartbeat_sent_timestamp(&self) {
        self.heartbeat_sent_timestamp.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Clock may have gone backwards")
                .as_millis() as u64,
            Ordering::Relaxed,
        );
    }

    #[allow(dead_code)]
    pub(crate) fn heartbeat_sent_timestamp(&self) -> u64 {
        self.heartbeat_sent_timestamp.load(Ordering::Relaxed)
    }

    pub(crate) fn set_heartbeat_received_timestamp(&self) {
        self.heartbeat_received_timestamp.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Clock may have gone backwards")
                .as_millis() as u64,
            Ordering::Relaxed,
        );
    }

    #[allow(dead_code)]
    pub(crate) fn heartbeat_received_timestamp(&self) -> u64 {
        self.heartbeat_received_timestamp.load(Ordering::Relaxed)
    }

    // TODO reduce to one atomic value ?
    pub fn is_synced(&self) -> bool {
        self.is_synced_threshold(0)
    }

    // TODO reduce to one atomic value ?
    pub fn is_synced_threshold(&self, threshold: u32) -> bool {
        *self.solid_milestone_index() >= (*self.latest_milestone_index()).saturating_sub(threshold)
    }

    pub(crate) fn has_data(&self, index: MilestoneIndex) -> bool {
        // +1 to allow for a little delay before a Heartbeat comes from a peer.
        index > self.pruned_index() && index <= self.solid_milestone_index() + MilestoneIndex(1)
    }

    pub(crate) fn maybe_has_data(&self, index: MilestoneIndex) -> bool {
        // +1 to allow for a little delay before a Heartbeat comes from a peer.
        index > self.pruned_index() && index <= self.latest_milestone_index() + MilestoneIndex(1)
    }
}
