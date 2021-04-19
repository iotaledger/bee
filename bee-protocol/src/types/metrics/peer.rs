// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that provides a type to hold metrics related to peers.

use std::sync::atomic::{AtomicU64, Ordering};

/// Holds metrics related to a peer.
#[derive(Default)]
pub struct PeerMetrics {
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
}

impl PeerMetrics {
    /// Returns the number of invalid messages of the `PeerMetrics`.
    pub fn invalid_messages(&self) -> u64 {
        self.invalid_messages.load(Ordering::Relaxed)
    }

    /// Increments the number of invalid messages of the `PeerMetrics`.
    pub fn invalid_messages_inc(&self) -> u64 {
        self.invalid_messages.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of new messages of the `PeerMetrics`.
    pub fn new_messages(&self) -> u64 {
        self.new_messages.load(Ordering::Relaxed)
    }

    /// Increments the number of new messages of the `PeerMetrics`.
    pub fn new_messages_inc(&self) -> u64 {
        self.new_messages.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of known messages of the `PeerMetrics`.
    pub fn known_messages(&self) -> u64 {
        self.known_messages.load(Ordering::Relaxed)
    }

    /// Increments the number of known messages of the `PeerMetrics`.
    pub fn known_messages_inc(&self) -> u64 {
        self.known_messages.fetch_add(1, Ordering::SeqCst)
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

    /// Returns the number of received messages of the `PeerMetrics`.
    pub fn messages_received(&self) -> u64 {
        self.messages_received.load(Ordering::Relaxed)
    }

    /// Increments the number of received messages of the `PeerMetrics`.
    pub fn messages_received_inc(&self) -> u64 {
        self.messages_received.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of received messages requests of the `PeerMetrics`.
    pub fn message_requests_received(&self) -> u64 {
        self.message_requests_received.load(Ordering::Relaxed)
    }

    /// Increments the number of received message requests of the `PeerMetrics`.
    pub fn message_requests_received_inc(&self) -> u64 {
        self.message_requests_received.fetch_add(1, Ordering::SeqCst)
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

    /// Returns the number of sent messages of the `PeerMetrics`.
    pub fn messages_sent(&self) -> u64 {
        self.messages_sent.load(Ordering::Relaxed)
    }

    /// Increments the number of sent messages of the `PeerMetrics`.
    pub fn messages_sent_inc(&self) -> u64 {
        self.messages_sent.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the number of sent message requests of the `PeerMetrics`.
    pub fn message_requests_sent(&self) -> u64 {
        self.message_requests_sent.load(Ordering::Relaxed)
    }

    /// Increments the number of sent message requests of the `PeerMetrics`.
    pub fn message_requests_sent_inc(&self) -> u64 {
        self.message_requests_sent.fetch_add(1, Ordering::SeqCst)
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
