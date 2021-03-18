// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::metrics::NodeMetrics,
    workers::{
        packets::{tlv_into_bytes, Heartbeat, Message as MessagePacket, MessageRequest, MilestoneRequest, Packet},
        peer::PeerManager,
    },
};

use bee_network::PeerId;

use log::warn;

use std::marker::PhantomData;

pub(crate) struct Sender<P: Packet> {
    marker: PhantomData<P>,
}

impl Sender<MilestoneRequest> {
    pub(crate) async fn send(peer_manager: &PeerManager, metrics: &NodeMetrics, id: &PeerId, packet: MilestoneRequest) {
        if let Some(peer) = peer_manager.get(id).await {
            if let Some(ref sender) = peer.1 {
                match sender.0.send(tlv_into_bytes(packet)) {
                    Ok(_) => {
                        (*peer).0.metrics().milestone_requests_sent_inc();
                        metrics.milestone_requests_sent_inc();
                    }
                    Err(e) => {
                        warn!("Sending MilestoneRequest to {} failed: {:?}.", id, e);
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

impl Sender<MessageRequest> {
    pub(crate) async fn send(peer_manager: &PeerManager, metrics: &NodeMetrics, id: &PeerId, packet: MessageRequest) {
        if let Some(peer) = peer_manager.get(id).await {
            if let Some(ref sender) = peer.1 {
                match sender.0.send(tlv_into_bytes(packet)) {
                    Ok(_) => {
                        (*peer).0.metrics().message_requests_sent_inc();
                        metrics.message_requests_sent_inc();
                    }
                    Err(e) => {
                        warn!("Sending MessageRequest to {} failed: {:?}.", id, e);
                    }
                }
            }
        }
    }
}

impl Sender<Heartbeat> {
    pub(crate) async fn send(peer_manager: &PeerManager, metrics: &NodeMetrics, id: &PeerId, packet: Heartbeat) {
        if let Some(peer) = peer_manager.get(id).await {
            if let Some(ref sender) = peer.1 {
                match sender.0.send(tlv_into_bytes(packet)) {
                    Ok(_) => {
                        (*peer).0.metrics().heartbeats_sent_inc();
                        (*peer).0.set_heartbeat_sent_timestamp();
                        metrics.heartbeats_sent_inc();
                    }
                    Err(e) => {
                        warn!("Sending Heartbeat to {} failed: {:?}.", id, e);
                    }
                }
            }
        }
    }
}
