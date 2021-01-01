// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    packet::{tlv_into_bytes, Heartbeat, Message as MessagePacket, MessageRequest, MilestoneRequest, Packet},
    peer::PeerManager,
    ProtocolMetrics,
};

use bee_network::{Command::SendMessage, NetworkController, PeerId};

use log::warn;

use std::marker::PhantomData;

pub(crate) struct Sender<P: Packet> {
    marker: PhantomData<P>,
}

impl Sender<MilestoneRequest> {
    pub(crate) fn send(
        network: &NetworkController,
        peer_manager: &PeerManager,
        metrics: &ProtocolMetrics,
        id: &PeerId,
        packet: MilestoneRequest,
    ) {
        if let Some(peer) = peer_manager.get(id) {
            match network.send(SendMessage {
                to: id.clone(),
                message: tlv_into_bytes(packet),
            }) {
                Ok(_) => {
                    peer.metrics().milestone_requests_sent_inc();
                    metrics.milestone_requests_sent_inc();
                }
                Err(e) => {
                    warn!("Sending MilestoneRequest to {} failed: {:?}.", id, e);
                }
            }
        }
    }
}

impl Sender<MessagePacket> {
    pub(crate) fn send(
        network: &NetworkController,
        peer_manager: &PeerManager,
        metrics: &ProtocolMetrics,
        id: &PeerId,
        packet: MessagePacket,
    ) {
        if let Some(peer) = peer_manager.get(id) {
            match network.send(SendMessage {
                to: id.clone(),
                message: tlv_into_bytes(packet),
            }) {
                Ok(_) => {
                    peer.metrics().messages_sent_inc();
                    metrics.messages_sent_inc();
                }
                Err(e) => {
                    warn!("Sending MessagePacket to {} failed: {:?}.", id, e);
                }
            }
        }
    }
}

impl Sender<MessageRequest> {
    pub(crate) fn send(
        network: &NetworkController,
        peer_manager: &PeerManager,
        metrics: &ProtocolMetrics,
        id: &PeerId,
        packet: MessageRequest,
    ) {
        if let Some(peer) = peer_manager.get(id) {
            match network.send(SendMessage {
                to: id.clone(),
                message: tlv_into_bytes(packet),
            }) {
                Ok(_) => {
                    peer.metrics().message_requests_sent_inc();
                    metrics.message_requests_sent_inc();
                }
                Err(e) => {
                    warn!("Sending MessageRequest to {} failed: {:?}.", id, e);
                }
            }
        }
    }
}

impl Sender<Heartbeat> {
    pub(crate) fn send(
        network: &NetworkController,
        peer_manager: &PeerManager,
        metrics: &ProtocolMetrics,
        id: &PeerId,
        packet: Heartbeat,
    ) {
        if let Some(peer) = peer_manager.get(id) {
            match network.send(SendMessage {
                to: id.clone(),
                message: tlv_into_bytes(packet),
            }) {
                Ok(_) => {
                    peer.metrics().heartbeats_sent_inc();
                    peer.set_heartbeat_sent_timestamp();
                    metrics.heartbeats_sent_inc();
                }
                Err(e) => {
                    warn!("Sending Heartbeat to {} failed: {:?}.", id, e);
                }
            }
        }
    }
}
