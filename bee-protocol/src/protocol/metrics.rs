// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct ProtocolMetrics {
    invalid_messages: AtomicU64,
    new_messages: AtomicU64,
    known_messages: AtomicU64,

    invalid_packets: AtomicU64,

    milestone_requests_received: AtomicU64,
    messages_received: AtomicU64,
    message_requests_received: AtomicU64,
    heartbeats_received: AtomicU64,

    milestone_requests_sent: AtomicU64,
    messages_sent: AtomicU64,
    message_requests_sent: AtomicU64,
    heartbeats_sent: AtomicU64,

    value_bundles: AtomicU64,
    non_value_bundles: AtomicU64,
    confirmed_bundles: AtomicU64,
    conflicting_bundles: AtomicU64,
}

impl ProtocolMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ProtocolMetrics {
    pub fn invalid_messages(&self) -> u64 {
        self.invalid_messages.load(Ordering::Relaxed)
    }

    pub(crate) fn invalid_messages_inc(&self) -> u64 {
        self.invalid_messages.fetch_add(1, Ordering::SeqCst)
    }

    pub fn new_messages(&self) -> u64 {
        self.new_messages.load(Ordering::Relaxed)
    }

    pub(crate) fn new_messages_inc(&self) -> u64 {
        self.new_messages.fetch_add(1, Ordering::SeqCst)
    }

    pub fn known_messages(&self) -> u64 {
        self.known_messages.load(Ordering::Relaxed)
    }

    pub(crate) fn known_messages_inc(&self) -> u64 {
        self.known_messages.fetch_add(1, Ordering::SeqCst)
    }

    pub fn invalid_packets(&self) -> u64 {
        self.invalid_packets.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn invalid_packets_inc(&self) -> u64 {
        self.invalid_packets.fetch_add(1, Ordering::SeqCst)
    }

    pub fn milestone_requests_received(&self) -> u64 {
        self.milestone_requests_received.load(Ordering::Relaxed)
    }

    pub(crate) fn milestone_requests_received_inc(&self) -> u64 {
        self.milestone_requests_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn messages_received(&self) -> u64 {
        self.messages_received.load(Ordering::Relaxed)
    }

    pub(crate) fn messages_received_inc(&self) -> u64 {
        self.messages_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn message_requests_received(&self) -> u64 {
        self.message_requests_received.load(Ordering::Relaxed)
    }

    pub(crate) fn message_requests_received_inc(&self) -> u64 {
        self.message_requests_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn heartbeats_received(&self) -> u64 {
        self.heartbeats_received.load(Ordering::Relaxed)
    }

    pub(crate) fn heartbeats_received_inc(&self) -> u64 {
        self.heartbeats_received.fetch_add(1, Ordering::SeqCst)
    }

    pub fn milestone_requests_sent(&self) -> u64 {
        self.milestone_requests_sent.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn milestone_requests_sent_inc(&self) -> u64 {
        self.milestone_requests_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn messages_sent(&self) -> u64 {
        self.messages_sent.load(Ordering::Relaxed)
    }

    pub(crate) fn messages_sent_inc(&self) -> u64 {
        self.messages_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn message_requests_sent(&self) -> u64 {
        self.message_requests_sent.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn message_requests_sent_inc(&self) -> u64 {
        self.message_requests_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn heartbeats_sent(&self) -> u64 {
        self.heartbeats_sent.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn heartbeats_sent_inc(&self) -> u64 {
        self.heartbeats_sent.fetch_add(1, Ordering::SeqCst)
    }

    pub fn value_bundles(&self) -> u64 {
        self.value_bundles.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn value_bundles_inc(&self) -> u64 {
        self.value_bundles.fetch_add(1, Ordering::SeqCst)
    }

    pub fn non_value_bundles(&self) -> u64 {
        self.non_value_bundles.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn non_value_bundles_inc(&self) -> u64 {
        self.non_value_bundles.fetch_add(1, Ordering::SeqCst)
    }

    pub fn confirmed_bundles(&self) -> u64 {
        self.confirmed_bundles.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn confirmed_bundles_inc(&self) -> u64 {
        self.confirmed_bundles.fetch_add(1, Ordering::SeqCst)
    }

    pub fn conflicting_bundles(&self) -> u64 {
        self.conflicting_bundles.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn conflicting_bundles_inc(&self) -> u64 {
        self.conflicting_bundles.fetch_add(1, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn protocol_metrics_messages() {
        let metrics = ProtocolMetrics::default();

        assert_eq!(metrics.invalid_messages(), 0);
        assert_eq!(metrics.new_messages(), 0);
        assert_eq!(metrics.known_messages(), 0);

        metrics.invalid_messages_inc();
        metrics.new_messages_inc();
        metrics.known_messages_inc();

        assert_eq!(metrics.invalid_messages(), 1);
        assert_eq!(metrics.new_messages(), 1);
        assert_eq!(metrics.known_messages(), 1);
    }

    #[test]
    fn protocol_metrics_packets_received() {
        let metrics = ProtocolMetrics::default();

        assert_eq!(metrics.invalid_packets(), 0);
        assert_eq!(metrics.milestone_requests_received(), 0);
        assert_eq!(metrics.messages_received(), 0);
        assert_eq!(metrics.message_requests_received(), 0);
        assert_eq!(metrics.heartbeats_received(), 0);

        metrics.invalid_packets_inc();
        metrics.milestone_requests_received_inc();
        metrics.messages_received_inc();
        metrics.message_requests_received_inc();
        metrics.heartbeats_received_inc();

        assert_eq!(metrics.invalid_packets(), 1);
        assert_eq!(metrics.milestone_requests_received(), 1);
        assert_eq!(metrics.messages_received(), 1);
        assert_eq!(metrics.message_requests_received(), 1);
        assert_eq!(metrics.heartbeats_received(), 1);
    }

    #[test]
    fn protocol_metrics_packets_sent() {
        let metrics = ProtocolMetrics::default();

        assert_eq!(metrics.milestone_requests_sent(), 0);
        assert_eq!(metrics.messages_sent(), 0);
        assert_eq!(metrics.message_requests_sent(), 0);
        assert_eq!(metrics.heartbeats_sent(), 0);

        metrics.milestone_requests_sent_inc();
        metrics.messages_sent_inc();
        metrics.message_requests_sent_inc();
        metrics.heartbeats_sent_inc();

        assert_eq!(metrics.milestone_requests_sent(), 1);
        assert_eq!(metrics.messages_sent(), 1);
        assert_eq!(metrics.message_requests_sent(), 1);
        assert_eq!(metrics.heartbeats_sent(), 1);
    }

    #[test]
    fn protocol_metrics_confirmation() {
        let metrics = ProtocolMetrics::default();

        assert_eq!(metrics.value_bundles(), 0);
        assert_eq!(metrics.non_value_bundles(), 0);
        assert_eq!(metrics.confirmed_bundles(), 0);
        assert_eq!(metrics.conflicting_bundles(), 0);

        metrics.value_bundles_inc();
        metrics.non_value_bundles_inc();
        metrics.confirmed_bundles_inc();
        metrics.conflicting_bundles_inc();

        assert_eq!(metrics.value_bundles(), 1);
        assert_eq!(metrics.non_value_bundles(), 1);
        assert_eq!(metrics.confirmed_bundles(), 1);
        assert_eq!(metrics.conflicting_bundles(), 1);
    }
}
