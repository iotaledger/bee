// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::marker::PhantomData;

use bee_gossip::PeerId;
use log::warn;

use crate::{
    types::metrics::NodeMetrics,
    workers::{
        packets::{tlv_to_bytes, HeartbeatPacket, MessagePacket, MessageRequestPacket, MilestoneRequestPacket, Packet},
        peer::PeerManager,
    },
};

pub(crate) struct Sender<P: Packet> {
    marker: PhantomData<P>,
}

impl Sender<MilestoneRequestPacket> {
    pub(crate) fn send(
        packet: &MilestoneRequestPacket,
        id: &PeerId,
        peer_manager: &PeerManager,
        metrics: &NodeMetrics,
    ) {
        peer_manager
            .get_map(id, |peer| {
                if let Some(ref sender) = peer.1 {
                    match sender.0.send(tlv_to_bytes(packet)) {
                        Ok(_) => {
                            peer.0.metrics().milestone_requests_sent_inc();
                            metrics.milestone_requests_sent_inc();
                        }
                        Err(e) => {
                            warn!("Sending MilestoneRequestPacket to {} failed: {:?}.", id, e);
                        }
                    }
                }
            })
            .unwrap_or_default()
    }
}

impl Sender<MessagePacket> {
    pub(crate) fn send(packet: &MessagePacket, id: &PeerId, peer_manager: &PeerManager, metrics: &NodeMetrics) {
        peer_manager
            .get_map(id, |peer| {
                if let Some(ref sender) = peer.1 {
                    match sender.0.send(tlv_to_bytes(packet)) {
                        Ok(_) => {
                            peer.0.metrics().messages_sent_inc();
                            metrics.messages_sent_inc();
                        }
                        Err(e) => {
                            warn!("Sending MessagePacket to {} failed: {:?}.", id, e);
                        }
                    }
                }
            })
            .unwrap_or_default()
    }
}

impl Sender<MessageRequestPacket> {
    pub(crate) fn send(packet: &MessageRequestPacket, id: &PeerId, peer_manager: &PeerManager, metrics: &NodeMetrics) {
        peer_manager
            .get_map(id, |peer| {
                if let Some(ref sender) = peer.1 {
                    match sender.0.send(tlv_to_bytes(packet)) {
                        Ok(_) => {
                            peer.0.metrics().message_requests_sent_inc();
                            metrics.message_requests_sent_inc();
                        }
                        Err(e) => {
                            warn!("Sending MessageRequestPacket to {} failed: {:?}.", id, e);
                        }
                    }
                }
            })
            .unwrap_or_default()
    }
}

impl Sender<HeartbeatPacket> {
    pub(crate) fn send(packet: &HeartbeatPacket, id: &PeerId, peer_manager: &PeerManager, metrics: &NodeMetrics) {
        peer_manager
            .get_map(id, |peer| {
                if let Some(ref sender) = peer.1 {
                    match sender.0.send(tlv_to_bytes(packet)) {
                        Ok(_) => {
                            peer.0.metrics().heartbeats_sent_inc();
                            peer.0.set_heartbeat_sent_timestamp();
                            metrics.heartbeats_sent_inc();
                        }
                        Err(e) => {
                            warn!("Sending HeartbeatPacket to {} failed: {:?}.", id, e);
                        }
                    }
                }
            })
            .unwrap_or_default();
    }
}
