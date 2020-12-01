// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    milestone::MilestoneIndex,
    packet::{tlv_from_bytes, Header, Heartbeat, Message, MessageRequest, MilestoneRequest, Packet},
    peer::Peer,
    protocol::Protocol,
    tangle::MsTangle,
    worker::{
        peer::message_handler::MessageHandler, HasherWorkerEvent, MessageResponderWorkerEvent,
        MilestoneRequesterWorkerEvent, MilestoneResponderWorkerEvent, RequestedMilestones,
    },
};

use bee_common::node::ResHandle;
use bee_storage::storage::Backend;

use futures::{channel::oneshot, future::FutureExt};
use log::{error, info, trace, warn};

use std::sync::Arc;

#[derive(Debug)]
pub(crate) enum PeerWorkerError {
    FailedSend,
}

pub struct PeerWorker {
    peer: Arc<Peer>,
    hasher: flume::Sender<HasherWorkerEvent>,
    message_responder: flume::Sender<MessageResponderWorkerEvent>,
    milestone_responder: flume::Sender<MilestoneResponderWorkerEvent>,
    milestone_requester: flume::Sender<MilestoneRequesterWorkerEvent>,
}

impl PeerWorker {
    pub(crate) fn new(
        peer: Arc<Peer>,
        hasher: flume::Sender<HasherWorkerEvent>,
        message_responder: flume::Sender<MessageResponderWorkerEvent>,
        milestone_responder: flume::Sender<MilestoneResponderWorkerEvent>,
        milestone_requester: flume::Sender<MilestoneRequesterWorkerEvent>,
    ) -> Self {
        Self {
            peer,
            hasher,
            message_responder,
            milestone_responder,
            milestone_requester,
        }
    }

    pub(crate) async fn run<B: Backend>(
        mut self,
        tangle: ResHandle<MsTangle<B>>,
        requested_milestones: ResHandle<RequestedMilestones>,
        receiver: flume::Receiver<Vec<u8>>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("[{}] Running.", self.peer.address);

        let receiver_fused = receiver.into_stream();
        let shutdown_fused = shutdown.fuse();

        let mut message_handler = MessageHandler::new(receiver_fused, shutdown_fused, self.peer.address.clone());

        //                 Protocol::send_heartbeat(
        //                     self.peer.id,
        //                     tangle.get_latest_solid_milestone_index(),
        //                     tangle.get_pruning_index(),
        //                     tangle.get_latest_milestone_index(),
        //                 );
        //

        Protocol::request_latest_milestone(
            &*tangle,
            &self.milestone_requester,
            &*requested_milestones,
            // TODO should be copy ?
            Some(self.peer.id.clone()),
        );

        while let Some((header, bytes)) = message_handler.fetch_message().await {
            if let Err(e) = self.process_message(&tangle, &header, bytes) {
                error!("[{}] Processing message failed: {:?}.", self.peer.address, e);
            }
        }

        info!("[{}] Stopped.", self.peer.address);

        Protocol::get().peer_manager.remove(&self.peer.id).await;
    }

    fn process_message<B: Backend>(
        &mut self,
        tangle: &MsTangle<B>,
        header: &Header,
        bytes: &[u8],
    ) -> Result<(), PeerWorkerError> {
        match header.packet_type {
            MilestoneRequest::ID => {
                trace!("[{}] Reading MilestoneRequest...", self.peer.address);
                match tlv_from_bytes::<MilestoneRequest>(&header, bytes) {
                    Ok(message) => {
                        self.milestone_responder
                            .send(MilestoneResponderWorkerEvent {
                                peer_id: self.peer.id.clone(),
                                request: message,
                            })
                            .map_err(|_| PeerWorkerError::FailedSend)?;

                        self.peer.metrics.milestone_requests_received_inc();
                        Protocol::get().metrics.milestone_requests_received_inc();
                    }
                    Err(e) => {
                        warn!("[{}] Reading MilestoneRequest failed: {:?}.", self.peer.address, e);

                        self.peer.metrics.invalid_messages_inc();
                        Protocol::get().metrics.invalid_messages_inc();
                    }
                }
            }
            Message::ID => {
                trace!("[{}] Reading Message...", self.peer.address);
                match tlv_from_bytes::<Message>(&header, bytes) {
                    Ok(message) => {
                        self.hasher
                            .send(HasherWorkerEvent {
                                from: Some(self.peer.id.clone()),
                                message_packet: message,
                                notifier: None,
                            })
                            .map_err(|_| PeerWorkerError::FailedSend)?;

                        self.peer.metrics.messages_received_inc();
                        Protocol::get().metrics.messages_received_inc();
                    }
                    Err(e) => {
                        warn!("[{}] Reading Message failed: {:?}.", self.peer.address, e);

                        self.peer.metrics.invalid_messages_inc();
                        Protocol::get().metrics.invalid_messages_inc();
                    }
                }
            }
            MessageRequest::ID => {
                trace!("[{}] Reading MessageRequest...", self.peer.address);
                match tlv_from_bytes::<MessageRequest>(&header, bytes) {
                    Ok(message) => {
                        self.message_responder
                            .send(MessageResponderWorkerEvent {
                                peer_id: self.peer.id.clone(),
                                request: message,
                            })
                            .map_err(|_| PeerWorkerError::FailedSend)?;

                        self.peer.metrics.message_requests_received_inc();
                        Protocol::get().metrics.message_requests_received_inc();
                    }
                    Err(e) => {
                        warn!("[{}] Reading MessageRequest failed: {:?}.", self.peer.address, e);

                        self.peer.metrics.invalid_messages_inc();
                        Protocol::get().metrics.invalid_messages_inc();
                    }
                }
            }
            Heartbeat::ID => {
                trace!("[{}] Reading Heartbeat...", self.peer.address);
                match tlv_from_bytes::<Heartbeat>(&header, bytes) {
                    Ok(message) => {
                        self.peer
                            .set_latest_solid_milestone_index(message.latest_solid_milestone_index.into());
                        self.peer.set_pruned_index(message.pruned_index.into());
                        self.peer
                            .set_latest_milestone_index(message.latest_milestone_index.into());
                        self.peer.set_connected_peers(message.connected_peers);
                        self.peer.set_synced_peers(message.synced_peers);

                        if !tangle.is_synced_threshold(2)
                            && !self
                                .peer
                                .has_data(MilestoneIndex(*tangle.get_latest_solid_milestone_index() + 1))
                        {
                            warn!("The peer {} can't help syncing.", self.peer.address);
                            // TODO drop if autopeered.
                        }

                        // Also drop connection if autopeered and we can't help it sync

                        self.peer.metrics.heartbeats_received_inc();
                        Protocol::get().metrics.heartbeats_received_inc();
                    }
                    Err(e) => {
                        warn!("[{}] Reading Heartbeat failed: {:?}.", self.peer.address, e);

                        self.peer.metrics.invalid_messages_inc();
                        Protocol::get().metrics.invalid_messages_inc();
                    }
                }
            }
            _ => {
                warn!(
                    "[{}] Ignoring unsupported message type: {}.",
                    self.peer.address, header.packet_type
                );

                self.peer.metrics.invalid_messages_inc();
                Protocol::get().metrics.invalid_messages_inc();
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
