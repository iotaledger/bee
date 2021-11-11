// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::metrics::NodeMetrics,
    workers::{
        packets::{
            tlv_into_bytes, HeartbeatPacket, MessagePacket, MessageRequestPacket, MilestoneRequestPacket, Packet,
        },
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
        peer_manager: &PeerManager,
        metrics: &NodeMetrics,
        id: &PeerId,
        packet: MilestoneRequestPacket,
    ) {
        if let Some(peer) = peer_manager.get(id).await {
            if let Some(ref sender) = peer.1 {
                match sender.0.send(tlv_into_bytes(packet)) {
                    Ok(_) => {
                        (*peer).0.metrics().milestone_requests_sent_inc();
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
    pub(crate) async fn send(peer_manager: &PeerManager, metrics: &NodeMetrics, id: &PeerId, packet: MessagePacket) {
        if let Some(peer) = peer_manager.get(id).await {
            if let Some(ref sender) = peer.1 {
                match sender.0.send(tlv_into_bytes(packet)) {
                    Ok(_) => {
                        (*peer).0.metrics().messages_sent_inc();
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
        peer_manager: &PeerManager,
        metrics: &NodeMetrics,
        id: &PeerId,
        packet: MessageRequestPacket,
    ) {
        if let Some(peer) = peer_manager.get(id).await {
            if let Some(ref sender) = peer.1 {
                match sender.0.send(tlv_into_bytes(packet)) {
                    Ok(_) => {
                        (*peer).0.metrics().message_requests_sent_inc();
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
    pub(crate) async fn send(peer_manager: &PeerManager, metrics: &NodeMetrics, id: &PeerId, packet: HeartbeatPacket) {
        if let Some(peer) = peer_manager.get(id).await {
            if let Some(ref sender) = peer.1 {
                match sender.0.send(tlv_into_bytes(packet)) {
                    Ok(_) => {
                        (*peer).0.metrics().heartbeats_sent_inc();
                        (*peer).0.set_heartbeat_sent_timestamp();
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
