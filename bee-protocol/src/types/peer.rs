// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides a type describing peers.

use crate::types::metrics::PeerMetrics;

use bee_message::milestone::MilestoneIndex;
use bee_network::{Multiaddr, PeerId, PeerInfo, PeerRelation};

use std::{
    sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const SYNCED_THRESHOLD: u32 = 2;

/// A type holding information related to a peer.
pub struct Peer {
    id: PeerId,
    info: PeerInfo,
    connected: AtomicBool,
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
    /// Creates a new `Peer`.
    pub fn new(id: PeerId, info: PeerInfo) -> Self {
        Self {
            id,
            info,
            connected: AtomicBool::new(false),
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

    /// Returns the identifier of the `Peer`.
    pub fn id(&self) -> &PeerId {
        &self.id
    }

    /// Returns the address of the `Peer`.
    pub fn address(&self) -> &Multiaddr {
        &self.info.address
    }

    /// Returns the alias of the `Peer`.
    pub fn alias(&self) -> &String {
        &self.info.alias
    }

    /// Returns the relationship kind of the `Peer`.
    pub fn relation(&self) -> PeerRelation {
        self.info.relation
    }

    /// Returns whether the `Peer` is connected or not.
    pub fn set_connected(&self, connected: bool) {
        self.connected.store(connected, Ordering::Relaxed);
    }

    /// Sets whether the `Peer` is connected or not.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Returns the metrics of the `Peer`.
    pub fn metrics(&self) -> &PeerMetrics {
        &self.metrics
    }

    /// Sets the solid milestone index of the `Peer`.
    pub fn set_solid_milestone_index(&self, index: MilestoneIndex) {
        self.solid_milestone_index.store(*index, Ordering::Relaxed);
    }

    /// Returns the solid milestone index of the `Peer`.
    pub fn solid_milestone_index(&self) -> MilestoneIndex {
        self.solid_milestone_index.load(Ordering::Relaxed).into()
    }

    /// Sets the pruned index of the `Peer`.
    pub fn set_pruned_index(&self, index: MilestoneIndex) {
        self.pruned_index.store(*index, Ordering::Relaxed);
    }

    /// Returns the pruned index of the `Peer`.
    pub fn pruned_index(&self) -> MilestoneIndex {
        self.pruned_index.load(Ordering::Relaxed).into()
    }

    /// Sets the latest milestone index of the `Peer`.
    pub fn set_latest_milestone_index(&self, index: MilestoneIndex) {
        self.latest_milestone_index.store(*index, Ordering::Relaxed);
    }

    /// Returns the latest milestone index of the `Peer`.
    pub fn latest_milestone_index(&self) -> MilestoneIndex {
        self.latest_milestone_index.load(Ordering::Relaxed).into()
    }

    /// Sets the number of connected peers of the `Peer`.
    pub fn set_connected_peers(&self, connected_peers: u8) {
        self.connected_peers.store(connected_peers, Ordering::Relaxed);
    }

    /// Returns the number of connected peers of the `Peer`.
    pub fn connected_peers(&self) -> u8 {
        self.connected_peers.load(Ordering::Relaxed)
    }

    /// Sets the number of synced peers of the `Peer`.
    pub fn set_synced_peers(&self, synced_peers: u8) {
        self.synced_peers.store(synced_peers, Ordering::Relaxed);
    }

    /// Returns the number of synced peers of the `Peer`.
    pub fn synced_peers(&self) -> u8 {
        self.synced_peers.load(Ordering::Relaxed)
    }

    /// Sets the timestamp of the last heartbeat sent by the `Peer`.
    pub fn set_heartbeat_sent_timestamp(&self) {
        self.heartbeat_sent_timestamp.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Clock may have gone backwards")
                .as_millis() as u64,
            Ordering::Relaxed,
        );
    }

    /// Returns the timestamp of the last heartbeat sent by the `Peer`.
    pub fn heartbeat_sent_timestamp(&self) -> u64 {
        self.heartbeat_sent_timestamp.load(Ordering::Relaxed)
    }

    /// Sets the timestamp of the last heartbeat received by the `Peer`.
    pub fn set_heartbeat_received_timestamp(&self) {
        self.heartbeat_received_timestamp.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Clock may have gone backwards")
                .as_millis() as u64,
            Ordering::Relaxed,
        );
    }

    /// Returns the timestamp of the last heartbeat received by the `Peer`.
    pub fn heartbeat_received_timestamp(&self) -> u64 {
        self.heartbeat_received_timestamp.load(Ordering::Relaxed)
    }

    /// Returns whether the `Peer` is synced or not.
    pub fn is_synced(&self) -> bool {
        self.is_synced_threshold(SYNCED_THRESHOLD)
    }

    /// Returns whether the `Peer` is synced with a threshold or not.
    pub fn is_synced_threshold(&self, threshold: u32) -> bool {
        *self.solid_milestone_index() >= (*self.latest_milestone_index()).saturating_sub(threshold)
    }

    /// Returns whether the `Peer` has the data referenced by a given milestone index.
    pub fn has_data(&self, index: MilestoneIndex) -> bool {
        // +1 to allow for a little delay before a Heartbeat comes from a peer.
        index > self.pruned_index() && index <= self.solid_milestone_index() + MilestoneIndex(1)
    }

    /// Returns whether the `Peer` may have the data referenced by a given milestone index.
    pub fn maybe_has_data(&self, index: MilestoneIndex) -> bool {
        // +1 to allow for a little delay before a Heartbeat comes from a peer.
        index > self.pruned_index() && index <= self.latest_milestone_index() + MilestoneIndex(1)
    }
}
