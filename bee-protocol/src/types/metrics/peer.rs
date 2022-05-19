// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that provides a type to hold metrics related to peers.

use std::sync::atomic::{AtomicU64, Ordering};

/// Holds metrics related to a peer.
#[derive(Default)]
pub struct PeerMetrics {
    invalid_blocks: AtomicU64,
    new_blocks: AtomicU64,
    known_blocks: AtomicU64,
    invalid_packets: AtomicU64,
    milestone_requests_received: AtomicU64,
    blocks_received: AtomicU64,
    block_requests_received: AtomicU64,
    heartbeats_received: AtomicU64,
    milestone_requests_sent: AtomicU64,
    blocks_sent: AtomicU64,
    block_requests_sent: AtomicU64,
    heartbeats_sent: AtomicU64,
}

impl PeerMetrics {
    /// Returns the number of invalid blocks of the `PeerMetrics`.
    pub fn invalid_blocks(&self) -> u64 {
        self.invalid_blocks.load(Ordering::Relaxed)
    }

    /// Increments the number of invalid blocks of the `PeerMetrics`.
    pub fn invalid_blocks_inc(&self) -> u64 {
        self.invalid_blocks.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of new blocks of the `PeerMetrics`.
    pub fn new_blocks(&self) -> u64 {
        self.new_blocks.load(Ordering::Relaxed)
    }

    /// Increments the number of new blocks of the `PeerMetrics`.
    pub fn new_blocks_inc(&self) -> u64 {
        self.new_blocks.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of known blocks of the `PeerMetrics`.
    pub fn known_blocks(&self) -> u64 {
        self.known_blocks.load(Ordering::Relaxed)
    }

    /// Increments the number of known blocks of the `PeerMetrics`.
    pub fn known_blocks_inc(&self) -> u64 {
        self.known_blocks.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of invalid packets of the `PeerMetrics`.
    pub fn invalid_packets(&self) -> u64 {
        self.invalid_packets.load(Ordering::Relaxed)
    }

    /// Increments the number of invalid packets of the `PeerMetrics`.
    pub fn invalid_packets_inc(&self) -> u64 {
        self.invalid_packets.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of received milestones requests of the `PeerMetrics`.
    pub fn milestone_requests_received(&self) -> u64 {
        self.milestone_requests_received.load(Ordering::Relaxed)
    }

    /// Increments the number of received milestone requests of the `PeerMetrics`.
    pub fn milestone_requests_received_inc(&self) -> u64 {
        self.milestone_requests_received.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of received blocks of the `PeerMetrics`.
    pub fn blocks_received(&self) -> u64 {
        self.blocks_received.load(Ordering::Relaxed)
    }

    /// Increments the number of received blocks of the `PeerMetrics`.
    pub fn blocks_received_inc(&self) -> u64 {
        self.blocks_received.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of received blocks requests of the `PeerMetrics`.
    pub fn block_requests_received(&self) -> u64 {
        self.block_requests_received.load(Ordering::Relaxed)
    }

    /// Increments the number of received block requests of the `PeerMetrics`.
    pub fn block_requests_received_inc(&self) -> u64 {
        self.block_requests_received.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of received heartbeats of the `PeerMetrics`.
    pub fn heartbeats_received(&self) -> u64 {
        self.heartbeats_received.load(Ordering::Relaxed)
    }

    /// Increments the number of received heartbeats of the `PeerMetrics`.
    pub fn heartbeats_received_inc(&self) -> u64 {
        self.heartbeats_received.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of sent milestone requests of the `PeerMetrics`.
    pub fn milestone_requests_sent(&self) -> u64 {
        self.milestone_requests_sent.load(Ordering::Relaxed)
    }

    /// Increments the number of sent milestone requests of the `PeerMetrics`.
    pub fn milestone_requests_sent_inc(&self) -> u64 {
        self.milestone_requests_sent.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of sent blocks of the `PeerMetrics`.
    pub fn blocks_sent(&self) -> u64 {
        self.blocks_sent.load(Ordering::Relaxed)
    }

    /// Increments the number of sent blocks of the `PeerMetrics`.
    pub fn blocks_sent_inc(&self) -> u64 {
        self.blocks_sent.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of sent block requests of the `PeerMetrics`.
    pub fn block_requests_sent(&self) -> u64 {
        self.block_requests_sent.load(Ordering::Relaxed)
    }

    /// Increments the number of sent block requests of the `PeerMetrics`.
    pub fn block_requests_sent_inc(&self) -> u64 {
        self.block_requests_sent.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of sent heartbeats of the `PeerMetrics`.
    pub fn heartbeats_sent(&self) -> u64 {
        self.heartbeats_sent.load(Ordering::Relaxed)
    }

    /// Increments the number of sent heartbeats of the `PeerMetrics`.
    pub fn heartbeats_sent_inc(&self) -> u64 {
        self.heartbeats_sent.fetch_add(1, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn peer_metrics_blocks() {
        let metrics = PeerMetrics::default();

        assert_eq!(metrics.invalid_blocks(), 0);
        assert_eq!(metrics.new_blocks(), 0);
        assert_eq!(metrics.known_blocks(), 0);

        metrics.invalid_blocks_inc();
        metrics.new_blocks_inc();
        metrics.known_blocks_inc();

        assert_eq!(metrics.invalid_blocks(), 1);
        assert_eq!(metrics.new_blocks(), 1);
        assert_eq!(metrics.known_blocks(), 1);
    }

    #[test]
    fn peer_metrics_packets_received() {
        let metrics = PeerMetrics::default();

        assert_eq!(metrics.invalid_packets(), 0);
        assert_eq!(metrics.milestone_requests_received(), 0);
        assert_eq!(metrics.blocks_received(), 0);
        assert_eq!(metrics.block_requests_received(), 0);
        assert_eq!(metrics.heartbeats_received(), 0);

        metrics.invalid_packets_inc();
        metrics.milestone_requests_received_inc();
        metrics.blocks_received_inc();
        metrics.block_requests_received_inc();
        metrics.heartbeats_received_inc();

        assert_eq!(metrics.invalid_packets(), 1);
        assert_eq!(metrics.milestone_requests_received(), 1);
        assert_eq!(metrics.blocks_received(), 1);
        assert_eq!(metrics.block_requests_received(), 1);
        assert_eq!(metrics.heartbeats_received(), 1);
    }

    #[test]
    fn peer_metrics_packets_sent() {
        let metrics = PeerMetrics::default();

        assert_eq!(metrics.milestone_requests_sent(), 0);
        assert_eq!(metrics.blocks_sent(), 0);
        assert_eq!(metrics.block_requests_sent(), 0);
        assert_eq!(metrics.heartbeats_sent(), 0);

        metrics.milestone_requests_sent_inc();
        metrics.blocks_sent_inc();
        metrics.block_requests_sent_inc();
        metrics.heartbeats_sent_inc();

        assert_eq!(metrics.milestone_requests_sent(), 1);
        assert_eq!(metrics.blocks_sent(), 1);
        assert_eq!(metrics.block_requests_sent(), 1);
        assert_eq!(metrics.heartbeats_sent(), 1);
    }
}
