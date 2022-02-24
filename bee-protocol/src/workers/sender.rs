// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::metrics::NodeMetrics,
    workers::{
        packets::{tlv_to_bytes, HeartbeatPacket, MessagePacket, MessageRequestPacket, MilestoneRequestPacket, Packet},
        peer::PeerManager,
    },
};

use bee_gossip::PeerId;

use log::warn;

use std::marker::PhantomData;

pub(crate) struct Sender<P: Packet> {
    marker: PhantomData<P>,
}

impl Sender<MilestoneRequestPacket> {
    pub(crate) async fn send(
        packet: &MilestoneRequestPacket,
        id: &PeerId,
        peer_manager: &PeerManager,
        metrics: &NodeMetrics,
    ) {
        if let Some(ref mut peer) = peer_manager.get_mut(id).await {
            if let Some(ref mut sender) = peer.1 {
                match sender.0.send(&tlv_to_bytes(packet)).await {
                    Ok(_) => {
                        peer.0.metrics().milestone_requests_sent_inc();
                        metrics.milestone_requests_sent_inc();
                    }
                    Err(e) => {
                        warn!("Sending MilestoneRequestPacket to {} failed: {:?}.", id, e);
                    }
                }
            }
        }
    }
}

impl Sender<MessagePacket> {
    pub(crate) async fn send(packet: &MessagePacket, id: &PeerId, peer_manager: &PeerManager, metrics: &NodeMetrics) {
        if let Some(ref mut peer) = peer_manager.get_mut(id).await {
            if let Some(ref mut sender) = peer.1 {
                match sender.0.send(&tlv_to_bytes(packet)).await {
                    Ok(_) => {
                        peer.0.metrics().messages_sent_inc();
                        metrics.messages_sent_inc();
                    }
                    Err(e) => {
                        warn!("Sending MessagePacket to {} failed: {:?}.", id, e);
                    }
                }
            }
        }
    }
}

impl Sender<MessageRequestPacket> {
    pub(crate) async fn send(
        packet: &MessageRequestPacket,
        id: &PeerId,
        peer_manager: &PeerManager,
        metrics: &NodeMetrics,
    ) {
        if let Some(ref mut peer) = peer_manager.get_mut(id).await {
            if let Some(ref mut sender) = peer.1 {
                match sender.0.send(&tlv_to_bytes(packet)).await {
                    Ok(_) => {
                        peer.0.metrics().message_requests_sent_inc();
                        metrics.message_requests_sent_inc();
                    }
                    Err(e) => {
                        warn!("Sending MessageRequestPacket to {} failed: {:?}.", id, e);
                    }
                }
            }
        }
    }
}

impl Sender<HeartbeatPacket> {
    pub(crate) async fn send(packet: &HeartbeatPacket, id: &PeerId, peer_manager: &PeerManager, metrics: &NodeMetrics) {
        if let Some(ref mut peer) = peer_manager.get_mut(id).await {
            if let Some(ref mut sender) = peer.1 {
                match sender.0.send(&tlv_to_bytes(packet)).await {
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
        }
    }
}
