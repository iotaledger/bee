// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct PeerMetrics {
    #[allow(dead_code)]
    invalid_messages: AtomicU64,
    #[allow(dead_code)]
    new_messages: AtomicU64,
    #[allow(dead_code)]
    known_messages: AtomicU64,

    invalid_packets: AtomicU64,

    milestone_requests_received: AtomicU64,
    messages_received: AtomicU64,
    message_requests_received: AtomicU64,
    heartbeats_received: AtomicU64,

    #[allow(dead_code)]
    milestone_requests_sent: AtomicU64,
    messages_sent: AtomicU64,
    #[allow(dead_code)]
    message_requests_sent: AtomicU64,
    #[allow(dead_code)]
    heartbeats_sent: AtomicU64,
}

impl PeerMetrics {
    #[allow(dead_code)]
    pub fn invalid_messages(&self) -> u64 {
        self.invalid_messages.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn invalid_messages_inc(&self) -> u64 {
        self.invalid_messages.fetch_add(1, Ordering::SeqCst)
    }

    #[allow(dead_code)]
    pub fn new_messages(&self) -> u64 {
        self.new_messages.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn new_messages_inc(&self) -> u64 {
        self.new_messages.fetch_add(1, Ordering::SeqCst)
    }

    #[allow(dead_code)]
    pub fn known_messages(&self) -> u64 {
        self.known_messages.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn known_messages_inc(&self) -> u64 {
        self.known_messages.fetch_add(1, Ordering::SeqCst)
    }

    #[allow(dead_code)]
    pub fn invalid_packets(&self) -> u64 {
        self.invalid_packets.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn invalid_packets_inc(&self) -> u64 {
        self.invalid_packets.fetch_add(1, Ordering::SeqCst)
    }

    #[allow(dead_code)]
    pub fn milestone_requests_received(&self) -> u64 {
        self.milestone_requests_received.load(Ordering::Relaxed)
    }

    pub(crate) fn milestone_requests_received_inc(&self) -> u64 {
        self.milestone_requests_received.fetch_add(1, Ordering::SeqCst)
    }

    #[allow(dead_code)]
    pub fn messages_received(&self) -> u64 {
        self.messages_received.load(Ordering::Relaxed)
    }

    pub(crate) fn messages_received_inc(&self) -> u64 {
        self.messages_received.fetch_add(1, Ordering::SeqCst)
    }

    #[allow(dead_code)]
    pub fn message_requests_received(&self) -> u64 {
        self.message_requests_received.load(Ordering::Relaxed)
    }

    pub(crate) fn message_requests_received_inc(&self) -> u64 {
        self.message_requests_received.fetch_add(1, Ordering::SeqCst)
    }

    #[allow(dead_code)]
    pub fn heartbeats_received(&self) -> u64 {
        self.heartbeats_received.load(Ordering::Relaxed)
    }

    pub(crate) fn heartbeats_received_inc(&self) -> u64 {
        self.heartbeats_received.fetch_add(1, Ordering::SeqCst)
    }

    #[allow(dead_code)]
    pub fn milestone_requests_sent(&self) -> u64 {
        self.milestone_requests_sent.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn milestone_requests_sent_inc(&self) -> u64 {
        self.milestone_requests_sent.fetch_add(1, Ordering::SeqCst)
    }

    #[allow(dead_code)]
    pub fn messages_sent(&self) -> u64 {
        self.messages_sent.load(Ordering::Relaxed)
    }

    pub(crate) fn messages_sent_inc(&self) -> u64 {
        self.messages_sent.fetch_add(1, Ordering::SeqCst)
    }

    #[allow(dead_code)]
    pub fn message_requests_sent(&self) -> u64 {
        self.message_requests_sent.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn message_requests_sent_inc(&self) -> u64 {
        self.message_requests_sent.fetch_add(1, Ordering::SeqCst)
    }

    #[allow(dead_code)]
    pub fn heartbeats_sent(&self) -> u64 {
        self.heartbeats_sent.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn heartbeats_sent_inc(&self) -> u64 {
        self.heartbeats_sent.fetch_add(1, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn peer_metrics_messages() {
        let metrics = PeerMetrics::default();

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
    fn peer_metrics_packets_received() {
        let metrics = PeerMetrics::default();

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
    fn peer_metrics_packets_sent() {
        let metrics = PeerMetrics::default();

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
}
