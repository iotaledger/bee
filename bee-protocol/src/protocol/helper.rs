// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    milestone::MilestoneIndex,
    packet::{tlv_into_bytes, Heartbeat, Message as MessagePacket, MessageRequest, MilestoneRequest, Packet},
    protocol::Protocol,
    tangle::MsTangle,
    worker::{MessageRequesterWorkerEvent, MilestoneRequesterWorkerEvent, RequestedMessages, RequestedMilestones},
};

use bee_message::MessageId;
use bee_network::{Command::SendMessage, Network, PeerId};
use bee_storage::storage::Backend;

use log::warn;

use std::marker::PhantomData;

pub(crate) struct Sender<P: Packet> {
    marker: PhantomData<P>,
}

macro_rules! implement_sender_worker {
    ($type:ty, $sender:tt, $incrementor:tt) => {
        impl Sender<$type> {
            pub(crate) fn send(network: &Network, id: &PeerId, packet: $type) {
                match network.unbounded_send(SendMessage {
                    to: id.clone(),
                    message: tlv_into_bytes(packet),
                }) {
                    Ok(_) => {
                        // self.peer.metrics.$incrementor();
                        // Protocol::get().metrics.$incrementor();
                    }
                    Err(e) => {
                        warn!("Sending {} to {} failed: {:?}.", stringify!($type), id, e);
                    }
                }
            }
        }
    };
}

implement_sender_worker!(MilestoneRequest, milestone_request, milestone_requests_sent_inc);
implement_sender_worker!(MessagePacket, message, messages_sent_inc);
implement_sender_worker!(MessageRequest, message_request, message_requests_sent_inc);
implement_sender_worker!(Heartbeat, heartbeat, heartbeats_sent_inc);

impl Protocol {
    // TODO move some functions to workers

    // MilestoneRequest

    pub(crate) fn request_milestone<B: Backend>(
        tangle: &MsTangle<B>,
        milestone_requester: &flume::Sender<MilestoneRequesterWorkerEvent>,
        requested_milestones: &RequestedMilestones,
        index: MilestoneIndex,
        to: Option<PeerId>,
    ) {
        if !requested_milestones.contains_key(&index) && !tangle.contains_milestone(index) {
            if let Err(e) = milestone_requester.send(MilestoneRequesterWorkerEvent(index, to)) {
                warn!("Requesting milestone failed: {}.", e);
            }
        }
    }

    pub(crate) fn request_latest_milestone<B: Backend>(
        tangle: &MsTangle<B>,
        milestone_requester: &flume::Sender<MilestoneRequesterWorkerEvent>,
        requested_milestones: &RequestedMilestones,
        to: Option<PeerId>,
    ) {
        Protocol::request_milestone(tangle, milestone_requester, requested_milestones, MilestoneIndex(0), to)
    }

    // MessageRequest

    pub(crate) async fn request_message<B: Backend>(
        tangle: &MsTangle<B>,
        message_requester: &flume::Sender<MessageRequesterWorkerEvent>,
        requested_messages: &RequestedMessages,
        message_id: MessageId,
        index: MilestoneIndex,
    ) {
        if !tangle.contains(&message_id).await
            && !tangle.is_solid_entry_point(&message_id)
            && !requested_messages.contains_key(&message_id)
        {
            if let Err(e) = message_requester.send(MessageRequesterWorkerEvent(message_id, index)) {
                warn!("Requesting message failed: {}.", e);
            }
        }
    }

    // Heartbeat

    pub fn send_heartbeat(
        network: &Network,
        to: PeerId,
        latest_solid_milestone_index: MilestoneIndex,
        pruning_milestone_index: MilestoneIndex,
        latest_milestone_index: MilestoneIndex,
    ) {
        Sender::<Heartbeat>::send(
            network,
            &to,
            Heartbeat::new(
                *latest_solid_milestone_index,
                *pruning_milestone_index,
                *latest_milestone_index,
                Protocol::get().peer_manager.connected_peers(),
                Protocol::get().peer_manager.synced_peers(),
            ),
        );
    }

    pub fn broadcast_heartbeat(
        network: &Network,
        latest_solid_milestone_index: MilestoneIndex,
        pruning_milestone_index: MilestoneIndex,
        latest_milestone_index: MilestoneIndex,
    ) {
        for entry in Protocol::get().peer_manager.peers.iter() {
            Protocol::send_heartbeat(
                network,
                entry.key().clone(),
                latest_solid_milestone_index,
                pruning_milestone_index,
                latest_milestone_index,
            );
        }
    }
}
