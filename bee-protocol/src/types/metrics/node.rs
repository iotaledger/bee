// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that provides a type to hold metrics related to nodes.

use std::sync::atomic::{AtomicU64, Ordering};

/// Holds metrics related to a node.
#[derive(Default, Debug)]
pub struct NodeMetrics {
    invalid_packets: AtomicU64,

    milestone_requests_received: AtomicU64,
    messages_received: AtomicU64,
    message_requests_received: AtomicU64,
    heartbeats_received: AtomicU64,

    milestone_requests_sent: AtomicU64,
    messages_sent: AtomicU64,
    message_requests_sent: AtomicU64,
    heartbeats_sent: AtomicU64,

    invalid_messages: AtomicU64,
    new_messages: AtomicU64,
    known_messages: AtomicU64,
    messages_average_latency: AtomicU64,

    referenced_messages: AtomicU64,
    excluded_no_transaction_messages: AtomicU64,
    excluded_conflicting_messages: AtomicU64,
    included_messages: AtomicU64,

    created_outputs: AtomicU64,
    consumed_outputs: AtomicU64,
    receipts: AtomicU64,

    transaction_payloads: AtomicU64,
    milestone_payloads: AtomicU64,
    indexation_payloads: AtomicU64,

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

    /// Returns the number of received messages of the `NodeMetrics`.
    pub fn messages_received(&self) -> u64 {
        self.messages_received.load(Ordering::Relaxed)
    }

    /// Increments the number of received messages of the `NodeMetrics`.
    pub fn messages_received_inc(&self) -> u64 {
        self.messages_received.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of received message requests of the `NodeMetrics`.
    pub fn message_requests_received(&self) -> u64 {
        self.message_requests_received.load(Ordering::Relaxed)
    }

    /// Increments the number of received message requests of the `NodeMetrics`.
    pub fn message_requests_received_inc(&self) -> u64 {
        self.message_requests_received.fetch_add(1, Ordering::SeqCst)
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

    /// Returns the number of sent messages of the `NodeMetrics`.
    pub fn messages_sent(&self) -> u64 {
        self.messages_sent.load(Ordering::Relaxed)
    }

    /// Increments the number of sent messages of the `NodeMetrics`.
    pub fn messages_sent_inc(&self) -> u64 {
        self.messages_sent.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of sent message requests of the `NodeMetrics`.
    pub fn message_requests_sent(&self) -> u64 {
        self.message_requests_sent.load(Ordering::Relaxed)
    }

    /// Increments the number of sent message requests of the `NodeMetrics`.
    pub fn message_requests_sent_inc(&self) -> u64 {
        self.message_requests_sent.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of sent heartbeats of the `NodeMetrics`.
    pub fn heartbeats_sent(&self) -> u64 {
        self.heartbeats_sent.load(Ordering::Relaxed)
    }

    /// Increments the number of sent heartbeats of the `NodeMetrics`.
    pub fn heartbeats_sent_inc(&self) -> u64 {
        self.heartbeats_sent.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of invalid messages of the `NodeMetrics`.
    pub fn invalid_messages(&self) -> u64 {
        self.invalid_messages.load(Ordering::Relaxed)
    }

    /// Increments the number of invalid messages of the `NodeMetrics`.
    pub fn invalid_messages_inc(&self) -> u64 {
        self.invalid_messages.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of new messages of the `NodeMetrics`.
    pub fn new_messages(&self) -> u64 {
        self.new_messages.load(Ordering::Relaxed)
    }

    /// Increments the number of new messages of the `NodeMetrics`.
    pub fn new_messages_inc(&self) -> u64 {
        self.new_messages.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of known messages of the `NodeMetrics`.
    pub fn known_messages(&self) -> u64 {
        self.known_messages.load(Ordering::Relaxed)
    }

    /// Increments the number of known messages of the `NodeMetrics`.
    pub fn known_messages_inc(&self) -> u64 {
        self.known_messages.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the average messages latency of the `NodeMetrics`.
    pub fn messages_average_latency(&self) -> u64 {
        self.messages_average_latency.load(Ordering::Relaxed)
    }

    /// Sets the average messages latency of the `NodeMetrics`
    pub fn messages_average_latency_set(&self, val: u64) {
        self.messages_average_latency.store(val, Ordering::Relaxed)
    }

    /// Returns the number of referenced messages of the `NodeMetrics`.
    pub fn referenced_messages(&self) -> u64 {
        self.referenced_messages.load(Ordering::Relaxed)
    }

    /// Increments the number of referenced messages of the `NodeMetrics`.
    pub fn referenced_messages_inc(&self, value: u64) -> u64 {
        self.referenced_messages.fetch_add(value, Ordering::SeqCst)
    }

    /// Returns the number of excluded messages - because without transaction - of the `NodeMetrics`.
    pub fn excluded_no_transaction_messages(&self) -> u64 {
        self.excluded_no_transaction_messages.load(Ordering::Relaxed)
    }

    /// Increments the number of excluded messages - because without transaction - of the `NodeMetrics`.
    pub fn excluded_no_transaction_messages_inc(&self, value: u64) -> u64 {
        self.excluded_no_transaction_messages.fetch_add(value, Ordering::SeqCst)
    }

    /// Returns the number of excluded messages - because conflicting - of the `NodeMetrics`.
    pub fn excluded_conflicting_messages(&self) -> u64 {
        self.excluded_conflicting_messages.load(Ordering::Relaxed)
    }

    /// Increments the number of excluded messages - because conflicting - of the `NodeMetrics`.
    pub fn excluded_conflicting_messages_inc(&self, value: u64) -> u64 {
        self.excluded_conflicting_messages.fetch_add(value, Ordering::SeqCst)
    }

    /// Returns the number of included messages of the `NodeMetrics`.
    pub fn included_messages(&self) -> u64 {
        self.included_messages.load(Ordering::Relaxed)
    }

    /// Increments the number of included messages of the `NodeMetrics`.
    pub fn included_messages_inc(&self, value: u64) -> u64 {
        self.included_messages.fetch_add(value, Ordering::SeqCst)
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

    /// Returns the number of indexation payloads of the `NodeMetrics`.
    pub fn indexation_payloads(&self) -> u64 {
        self.indexation_payloads.load(Ordering::Relaxed)
    }

    /// Increments the number of indexation payloads of the `NodeMetrics`.
    pub fn indexation_payload_inc(&self, value: u64) -> u64 {
        self.indexation_payloads.fetch_add(value, Ordering::SeqCst)
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
        assert_eq!(metrics.messages_received(), 0);
        assert_eq!(metrics.message_requests_received(), 0);
        assert_eq!(metrics.heartbeats_received(), 0);
        assert_eq!(metrics.milestone_requests_sent(), 0);
        assert_eq!(metrics.messages_sent(), 0);
        assert_eq!(metrics.message_requests_sent(), 0);
        assert_eq!(metrics.heartbeats_sent(), 0);
        assert_eq!(metrics.invalid_messages(), 0);
        assert_eq!(metrics.new_messages(), 0);
        assert_eq!(metrics.known_messages(), 0);
        assert_eq!(metrics.messages_average_latency(), 0);
        assert_eq!(metrics.referenced_messages(), 0);
        assert_eq!(metrics.excluded_no_transaction_messages(), 0);
        assert_eq!(metrics.excluded_conflicting_messages(), 0);
        assert_eq!(metrics.included_messages(), 0);
        assert_eq!(metrics.created_outputs(), 0);
        assert_eq!(metrics.consumed_outputs(), 0);
        assert_eq!(metrics.receipts(), 0);
        assert_eq!(metrics.transaction_payloads(), 0);
        assert_eq!(metrics.milestone_payloads(), 0);
        assert_eq!(metrics.indexation_payloads(), 0);
        assert_eq!(metrics.snapshots(), 0);
        assert_eq!(metrics.prunings(), 0);

        metrics.invalid_packets_inc();
        metrics.milestone_requests_received_inc();
        metrics.messages_received_inc();
        metrics.message_requests_received_inc();
        metrics.heartbeats_received_inc();
        metrics.milestone_requests_sent_inc();
        metrics.messages_sent_inc();
        metrics.message_requests_sent_inc();
        metrics.heartbeats_sent_inc();
        metrics.invalid_messages_inc();
        metrics.new_messages_inc();
        metrics.known_messages_inc();
        metrics.messages_average_latency_set(42);
        metrics.referenced_messages_inc(1);
        metrics.excluded_no_transaction_messages_inc(1);
        metrics.excluded_conflicting_messages_inc(1);
        metrics.included_messages_inc(1);
        metrics.created_outputs_inc(1);
        metrics.consumed_outputs_inc(1);
        metrics.receipts_inc(1);
        metrics.transaction_payloads_inc(1);
        metrics.milestone_payloads_inc(1);
        metrics.indexation_payload_inc(1);
        metrics.snapshots_inc(1);
        metrics.prunings_inc(1);

        assert_eq!(metrics.invalid_packets(), 1);
        assert_eq!(metrics.milestone_requests_received(), 1);
        assert_eq!(metrics.messages_received(), 1);
        assert_eq!(metrics.message_requests_received(), 1);
        assert_eq!(metrics.heartbeats_received(), 1);
        assert_eq!(metrics.milestone_requests_sent(), 1);
        assert_eq!(metrics.messages_sent(), 1);
        assert_eq!(metrics.message_requests_sent(), 1);
        assert_eq!(metrics.heartbeats_sent(), 1);
        assert_eq!(metrics.invalid_messages(), 1);
        assert_eq!(metrics.new_messages(), 1);
        assert_eq!(metrics.known_messages(), 1);
        assert_eq!(metrics.messages_average_latency(), 42);
        assert_eq!(metrics.referenced_messages(), 1);
        assert_eq!(metrics.excluded_no_transaction_messages(), 1);
        assert_eq!(metrics.excluded_conflicting_messages(), 1);
        assert_eq!(metrics.included_messages(), 1);
        assert_eq!(metrics.created_outputs(), 1);
        assert_eq!(metrics.consumed_outputs(), 1);
        assert_eq!(metrics.receipts(), 1);
        assert_eq!(metrics.transaction_payloads(), 1);
        assert_eq!(metrics.milestone_payloads(), 1);
        assert_eq!(metrics.indexation_payloads(), 1);
        assert_eq!(metrics.snapshots(), 1);
        assert_eq!(metrics.prunings(), 1);
    }
}
