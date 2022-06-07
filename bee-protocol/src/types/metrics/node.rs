// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that provides a type to hold metrics related to nodes.

use std::sync::atomic::{AtomicU64, Ordering};

/// Holds metrics related to a node.
#[derive(Default, Debug)]
pub struct NodeMetrics {
    invalid_packets: AtomicU64,

    milestone_requests_received: AtomicU64,
    blocks_received: AtomicU64,
    block_requests_received: AtomicU64,
    heartbeats_received: AtomicU64,

    milestone_requests_sent: AtomicU64,
    blocks_sent: AtomicU64,
    block_requests_sent: AtomicU64,
    heartbeats_sent: AtomicU64,

    invalid_blocks: AtomicU64,
    new_blocks: AtomicU64,
    known_blocks: AtomicU64,
    blocks_average_latency: AtomicU64,

    referenced_blocks: AtomicU64,
    excluded_no_transaction_blocks: AtomicU64,
    excluded_conflicting_blocks: AtomicU64,
    included_blocks: AtomicU64,

    created_outputs: AtomicU64,
    consumed_outputs: AtomicU64,
    receipts: AtomicU64,

    transaction_payloads: AtomicU64,
    milestone_payloads: AtomicU64,
    tagged_data_payloads: AtomicU64,

    snapshots: AtomicU64,
    prunings: AtomicU64,
}

impl NodeMetrics {
    /// Creates a new `NodeMetrics`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl NodeMetrics {
    /// Returns the number of invalid packets of the `NodeMetrics`.
    pub fn invalid_packets(&self) -> u64 {
        self.invalid_packets.load(Ordering::Relaxed)
    }

    /// Increments the number of invalid packets of the `NodeMetrics`.
    pub fn invalid_packets_inc(&self) -> u64 {
        self.invalid_packets.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of received milestone requests of the `NodeMetrics`.
    pub fn milestone_requests_received(&self) -> u64 {
        self.milestone_requests_received.load(Ordering::Relaxed)
    }

    /// Increments the number of received milestone requests of the `NodeMetrics`.
    pub fn milestone_requests_received_inc(&self) -> u64 {
        self.milestone_requests_received.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of received blocks of the `NodeMetrics`.
    pub fn blocks_received(&self) -> u64 {
        self.blocks_received.load(Ordering::Relaxed)
    }

    /// Increments the number of received blocks of the `NodeMetrics`.
    pub fn blocks_received_inc(&self) -> u64 {
        self.blocks_received.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of received block requests of the `NodeMetrics`.
    pub fn block_requests_received(&self) -> u64 {
        self.block_requests_received.load(Ordering::Relaxed)
    }

    /// Increments the number of received block requests of the `NodeMetrics`.
    pub fn block_requests_received_inc(&self) -> u64 {
        self.block_requests_received.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of received heartbeats of the `NodeMetrics`.
    pub fn heartbeats_received(&self) -> u64 {
        self.heartbeats_received.load(Ordering::Relaxed)
    }

    /// Increments the number of received heartbeats of the `NodeMetrics`.
    pub fn heartbeats_received_inc(&self) -> u64 {
        self.heartbeats_received.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of sent milestone requests of the `NodeMetrics`.
    pub fn milestone_requests_sent(&self) -> u64 {
        self.milestone_requests_sent.load(Ordering::Relaxed)
    }

    /// Increments the number of sent milestone requests of the `NodeMetrics`.
    pub fn milestone_requests_sent_inc(&self) -> u64 {
        self.milestone_requests_sent.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of sent blocks of the `NodeMetrics`.
    pub fn blocks_sent(&self) -> u64 {
        self.blocks_sent.load(Ordering::Relaxed)
    }

    /// Increments the number of sent blocks of the `NodeMetrics`.
    pub fn blocks_sent_inc(&self) -> u64 {
        self.blocks_sent.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of sent block requests of the `NodeMetrics`.
    pub fn block_requests_sent(&self) -> u64 {
        self.block_requests_sent.load(Ordering::Relaxed)
    }

    /// Increments the number of sent block requests of the `NodeMetrics`.
    pub fn block_requests_sent_inc(&self) -> u64 {
        self.block_requests_sent.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of sent heartbeats of the `NodeMetrics`.
    pub fn heartbeats_sent(&self) -> u64 {
        self.heartbeats_sent.load(Ordering::Relaxed)
    }

    /// Increments the number of sent heartbeats of the `NodeMetrics`.
    pub fn heartbeats_sent_inc(&self) -> u64 {
        self.heartbeats_sent.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of invalid blocks of the `NodeMetrics`.
    pub fn invalid_blocks(&self) -> u64 {
        self.invalid_blocks.load(Ordering::Relaxed)
    }

    /// Increments the number of invalid blocks of the `NodeMetrics`.
    pub fn invalid_blocks_inc(&self) -> u64 {
        self.invalid_blocks.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of new blocks of the `NodeMetrics`.
    pub fn new_blocks(&self) -> u64 {
        self.new_blocks.load(Ordering::Relaxed)
    }

    /// Increments the number of new blocks of the `NodeMetrics`.
    pub fn new_blocks_inc(&self) -> u64 {
        self.new_blocks.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of known blocks of the `NodeMetrics`.
    pub fn known_blocks(&self) -> u64 {
        self.known_blocks.load(Ordering::Relaxed)
    }

    /// Increments the number of known blocks of the `NodeMetrics`.
    pub fn known_blocks_inc(&self) -> u64 {
        self.known_blocks.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the average blocks latency of the `NodeMetrics`.
    pub fn blocks_average_latency(&self) -> u64 {
        self.blocks_average_latency.load(Ordering::Relaxed)
    }

    /// Sets the average blocks latency of the `NodeMetrics`
    pub fn blocks_average_latency_set(&self, val: u64) {
        self.blocks_average_latency.store(val, Ordering::Relaxed)
    }

    /// Returns the number of referenced blocks of the `NodeMetrics`.
    pub fn referenced_blocks(&self) -> u64 {
        self.referenced_blocks.load(Ordering::Relaxed)
    }

    /// Increments the number of referenced blocks of the `NodeMetrics`.
    pub fn referenced_blocks_inc(&self, value: u64) -> u64 {
        self.referenced_blocks.fetch_add(value, Ordering::SeqCst)
    }

    /// Returns the number of excluded blocks - because without transaction - of the `NodeMetrics`.
    pub fn excluded_no_transaction_blocks(&self) -> u64 {
        self.excluded_no_transaction_blocks.load(Ordering::Relaxed)
    }

    /// Increments the number of excluded blocks - because without transaction - of the `NodeMetrics`.
    pub fn excluded_no_transaction_blocks_inc(&self, value: u64) -> u64 {
        self.excluded_no_transaction_blocks.fetch_add(value, Ordering::SeqCst)
    }

    /// Returns the number of excluded blocks - because conflicting - of the `NodeMetrics`.
    pub fn excluded_conflicting_blocks(&self) -> u64 {
        self.excluded_conflicting_blocks.load(Ordering::Relaxed)
    }

    /// Increments the number of excluded blocks - because conflicting - of the `NodeMetrics`.
    pub fn excluded_conflicting_blocks_inc(&self, value: u64) -> u64 {
        self.excluded_conflicting_blocks.fetch_add(value, Ordering::SeqCst)
    }

    /// Returns the number of included blocks of the `NodeMetrics`.
    pub fn included_blocks(&self) -> u64 {
        self.included_blocks.load(Ordering::Relaxed)
    }

    /// Increments the number of included blocks of the `NodeMetrics`.
    pub fn included_blocks_inc(&self, value: u64) -> u64 {
        self.included_blocks.fetch_add(value, Ordering::SeqCst)
    }

    /// Returns the number of created outputs of the `NodeMetrics`.
    pub fn created_outputs(&self) -> u64 {
        self.created_outputs.load(Ordering::Relaxed)
    }

    /// Increments the number of created outputs of the `NodeMetrics`.
    pub fn created_outputs_inc(&self, value: u64) -> u64 {
        self.created_outputs.fetch_add(value, Ordering::SeqCst)
    }

    /// Returns the number of consumed outputs of the `NodeMetrics`.
    pub fn consumed_outputs(&self) -> u64 {
        self.consumed_outputs.load(Ordering::Relaxed)
    }

    /// Increments the number of consumed outputs of the `NodeMetrics`.
    pub fn consumed_outputs_inc(&self, value: u64) -> u64 {
        self.consumed_outputs.fetch_add(value, Ordering::SeqCst)
    }

    /// Returns the number of receipts of the `NodeMetrics`.
    pub fn receipts(&self) -> u64 {
        self.receipts.load(Ordering::Relaxed)
    }

    /// Increments the number of receipts of the `NodeMetrics`.
    pub fn receipts_inc(&self, value: u64) -> u64 {
        self.receipts.fetch_add(value, Ordering::SeqCst)
    }

    /// Returns the number of transaction payloads of the `NodeMetrics`.
    pub fn transaction_payloads(&self) -> u64 {
        self.transaction_payloads.load(Ordering::Relaxed)
    }

    /// Increments the number of transaction payloads of the `NodeMetrics`.
    pub fn transaction_payloads_inc(&self, value: u64) -> u64 {
        self.transaction_payloads.fetch_add(value, Ordering::SeqCst)
    }

    /// Returns the number of milestone payloads of the `NodeMetrics`.
    pub fn milestone_payloads(&self) -> u64 {
        self.milestone_payloads.load(Ordering::Relaxed)
    }

    /// Increments the number of milestone payloads of the `NodeMetrics`.
    pub fn milestone_payloads_inc(&self, value: u64) -> u64 {
        self.milestone_payloads.fetch_add(value, Ordering::SeqCst)
    }

    /// Returns the number of tagged data payloads of the `NodeMetrics`.
    pub fn tagged_data_payloads(&self) -> u64 {
        self.tagged_data_payloads.load(Ordering::Relaxed)
    }

    /// Increments the number of tagged data payloads of the `NodeMetrics`.
    pub fn tagged_data_payload_inc(&self, value: u64) -> u64 {
        self.tagged_data_payloads.fetch_add(value, Ordering::SeqCst)
    }

    /// Returns the number of snapshots of the `NodeMetrics`.
    pub fn snapshots(&self) -> u64 {
        self.snapshots.load(Ordering::Relaxed)
    }

    /// Increments the number of snapshots of the `NodeMetrics`.
    pub fn snapshots_inc(&self, value: u64) -> u64 {
        self.snapshots.fetch_add(value, Ordering::SeqCst)
    }

    /// Returns the number of prunings of the `NodeMetrics`.
    pub fn prunings(&self) -> u64 {
        self.prunings.load(Ordering::Relaxed)
    }

    /// Increments the number of prunings of the `NodeMetrics`.
    pub fn prunings_inc(&self, value: u64) -> u64 {
        self.prunings.fetch_add(value, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn protocol_metrics() {
        let metrics = NodeMetrics::default();

        assert_eq!(metrics.invalid_packets(), 0);
        assert_eq!(metrics.milestone_requests_received(), 0);
        assert_eq!(metrics.blocks_received(), 0);
        assert_eq!(metrics.block_requests_received(), 0);
        assert_eq!(metrics.heartbeats_received(), 0);
        assert_eq!(metrics.milestone_requests_sent(), 0);
        assert_eq!(metrics.blocks_sent(), 0);
        assert_eq!(metrics.block_requests_sent(), 0);
        assert_eq!(metrics.heartbeats_sent(), 0);
        assert_eq!(metrics.invalid_blocks(), 0);
        assert_eq!(metrics.new_blocks(), 0);
        assert_eq!(metrics.known_blocks(), 0);
        assert_eq!(metrics.blocks_average_latency(), 0);
        assert_eq!(metrics.referenced_blocks(), 0);
        assert_eq!(metrics.excluded_no_transaction_blocks(), 0);
        assert_eq!(metrics.excluded_conflicting_blocks(), 0);
        assert_eq!(metrics.included_blocks(), 0);
        assert_eq!(metrics.created_outputs(), 0);
        assert_eq!(metrics.consumed_outputs(), 0);
        assert_eq!(metrics.receipts(), 0);
        assert_eq!(metrics.transaction_payloads(), 0);
        assert_eq!(metrics.milestone_payloads(), 0);
        assert_eq!(metrics.tagged_data_payloads(), 0);
        assert_eq!(metrics.snapshots(), 0);
        assert_eq!(metrics.prunings(), 0);

        metrics.invalid_packets_inc();
        metrics.milestone_requests_received_inc();
        metrics.blocks_received_inc();
        metrics.block_requests_received_inc();
        metrics.heartbeats_received_inc();
        metrics.milestone_requests_sent_inc();
        metrics.blocks_sent_inc();
        metrics.block_requests_sent_inc();
        metrics.heartbeats_sent_inc();
        metrics.invalid_blocks_inc();
        metrics.new_blocks_inc();
        metrics.known_blocks_inc();
        metrics.blocks_average_latency_set(42);
        metrics.referenced_blocks_inc(1);
        metrics.excluded_no_transaction_blocks_inc(1);
        metrics.excluded_conflicting_blocks_inc(1);
        metrics.included_blocks_inc(1);
        metrics.created_outputs_inc(1);
        metrics.consumed_outputs_inc(1);
        metrics.receipts_inc(1);
        metrics.transaction_payloads_inc(1);
        metrics.milestone_payloads_inc(1);
        metrics.tagged_data_payload_inc(1);
        metrics.snapshots_inc(1);
        metrics.prunings_inc(1);

        assert_eq!(metrics.invalid_packets(), 1);
        assert_eq!(metrics.milestone_requests_received(), 1);
        assert_eq!(metrics.blocks_received(), 1);
        assert_eq!(metrics.block_requests_received(), 1);
        assert_eq!(metrics.heartbeats_received(), 1);
        assert_eq!(metrics.milestone_requests_sent(), 1);
        assert_eq!(metrics.blocks_sent(), 1);
        assert_eq!(metrics.block_requests_sent(), 1);
        assert_eq!(metrics.heartbeats_sent(), 1);
        assert_eq!(metrics.invalid_blocks(), 1);
        assert_eq!(metrics.new_blocks(), 1);
        assert_eq!(metrics.known_blocks(), 1);
        assert_eq!(metrics.blocks_average_latency(), 42);
        assert_eq!(metrics.referenced_blocks(), 1);
        assert_eq!(metrics.excluded_no_transaction_blocks(), 1);
        assert_eq!(metrics.excluded_conflicting_blocks(), 1);
        assert_eq!(metrics.included_blocks(), 1);
        assert_eq!(metrics.created_outputs(), 1);
        assert_eq!(metrics.consumed_outputs(), 1);
        assert_eq!(metrics.receipts(), 1);
        assert_eq!(metrics.transaction_payloads(), 1);
        assert_eq!(metrics.milestone_payloads(), 1);
        assert_eq!(metrics.tagged_data_payloads(), 1);
        assert_eq!(metrics.snapshots(), 1);
        assert_eq!(metrics.prunings(), 1);
    }
}
