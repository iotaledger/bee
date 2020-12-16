// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    helper,
    milestone::MilestoneIndex,
    packet::{tlv_from_bytes, Header, Heartbeat, Message, MessageRequest, MilestoneRequest, Packet, TlvError},
    peer::{Peer, PeerManager},
    storage::Backend,
    tangle::MsTangle,
    worker::{
        peer::packet_handler::PacketHandler, HasherWorkerEvent, MessageResponderWorkerEvent,
        MilestoneRequesterWorkerEvent, MilestoneResponderWorkerEvent, RequestedMilestones,
    },
    ProtocolMetrics,
};

use bee_common_pt2::node::ResHandle;

use futures::{channel::oneshot, future::FutureExt};
use log::{error, info, trace, warn};
use tokio::sync::mpsc;

use std::sync::Arc;

#[derive(Debug)]
pub(crate) enum Error {
    UnsupportedPacketType(u8),
    TlvError(TlvError),
    FailedSend,
}

impl From<TlvError> for Error {
    fn from(error: TlvError) -> Self {
        Error::TlvError(error)
    }
}

pub struct PeerWorker {
    peer: Arc<Peer>,
    metrics: ResHandle<ProtocolMetrics>,
    peer_manager: ResHandle<PeerManager>,
    hasher: mpsc::UnboundedSender<HasherWorkerEvent>,
    message_responder: mpsc::UnboundedSender<MessageResponderWorkerEvent>,
    milestone_responder: mpsc::UnboundedSender<MilestoneResponderWorkerEvent>,
    milestone_requester: mpsc::UnboundedSender<MilestoneRequesterWorkerEvent>,
}

impl PeerWorker {
    pub(crate) fn new(
        peer: Arc<Peer>,
        metrics: ResHandle<ProtocolMetrics>,
        peer_manager: ResHandle<PeerManager>,
        hasher: mpsc::UnboundedSender<HasherWorkerEvent>,
        message_responder: mpsc::UnboundedSender<MessageResponderWorkerEvent>,
        milestone_responder: mpsc::UnboundedSender<MilestoneResponderWorkerEvent>,
        milestone_requester: mpsc::UnboundedSender<MilestoneRequesterWorkerEvent>,
    ) -> Self {
        Self {
            peer,
            metrics,
            peer_manager,
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
        receiver: mpsc::UnboundedReceiver<Vec<u8>>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("[{}] Running.", self.peer.address());

        let shutdown_fused = shutdown.fuse();

        let mut packet_handler = PacketHandler::new(receiver, shutdown_fused, self.peer.address().clone());

        //                 Protocol::send_heartbeat(
        //                     self.peer.id(),
        //                     tangle.get_latest_solid_milestone_index(),
        //                     tangle.get_pruning_index(),
        //                     tangle.get_latest_milestone_index(),
        //                 );
        //

        helper::request_latest_milestone(
            &*tangle,
            &self.milestone_requester,
            &*requested_milestones,
            // TODO should be copy ?
            Some(self.peer.id().clone()),
        );

        // TODO is this needed ?
        let tangle = tangle.into_weak();

        while let Some((header, bytes)) = packet_handler.fetch_packet().await {
            let tangle = tangle.upgrade().expect("Needed Tangle resource but it was removed");

            if let Err(e) = self.process_packet(&tangle, &header, bytes) {
                error!("[{}] Processing packet failed: {:?}.", self.peer.address(), e);
                self.peer.metrics().invalid_packets_inc();
                self.metrics.invalid_packets_inc();
            }
        }

        info!("[{}] Stopped.", self.peer.address());

        self.peer_manager.remove(&self.peer.id()).await;
    }

    fn process_packet<B: Backend>(&mut self, tangle: &MsTangle<B>, header: &Header, bytes: &[u8]) -> Result<(), Error> {
        match header.packet_type {
            MilestoneRequest::ID => {
                trace!("[{}] Reading MilestoneRequest...", self.peer.address());

                let packet = tlv_from_bytes::<MilestoneRequest>(&header, bytes)?;

                self.milestone_responder
                    .send(MilestoneResponderWorkerEvent {
                        peer_id: self.peer.id().clone(),
                        request: packet,
                    })
                    .map_err(|_| Error::FailedSend)?;

                self.peer.metrics().milestone_requests_received_inc();
                self.metrics.milestone_requests_received_inc();
            }
            Message::ID => {
                trace!("[{}] Reading Message...", self.peer.address());

                let packet = tlv_from_bytes::<Message>(&header, bytes)?;

                self.hasher
                    .send(HasherWorkerEvent {
                        from: Some(self.peer.id().clone()),
                        message_packet: packet,
                        notifier: None,
                    })
                    .map_err(|_| Error::FailedSend)?;

                self.peer.metrics().messages_received_inc();
                self.metrics.messages_received_inc();
            }
            MessageRequest::ID => {
                trace!("[{}] Reading MessageRequest...", self.peer.address());

                let packet = tlv_from_bytes::<MessageRequest>(&header, bytes)?;

                self.message_responder
                    .send(MessageResponderWorkerEvent {
                        peer_id: self.peer.id().clone(),
                        request: packet,
                    })
                    .map_err(|_| Error::FailedSend)?;

                self.peer.metrics().message_requests_received_inc();
                self.metrics.message_requests_received_inc();
            }
            Heartbeat::ID => {
                trace!("[{}] Reading Heartbeat...", self.peer.address());

                let packet = tlv_from_bytes::<Heartbeat>(&header, bytes)?;

                self.peer
                    .set_latest_solid_milestone_index(packet.latest_solid_milestone_index.into());
                self.peer.set_pruned_index(packet.pruned_index.into());
                self.peer
                    .set_latest_milestone_index(packet.latest_milestone_index.into());
                self.peer.set_connected_peers(packet.connected_peers);
                self.peer.set_synced_peers(packet.synced_peers);
                self.peer.set_heartbeat_received_timestamp();

                if !tangle.is_synced_threshold(2)
                    && !self
                        .peer
                        .has_data(MilestoneIndex(*tangle.get_latest_solid_milestone_index() + 1))
                {
                    warn!(
                        "The peer {} can't help syncing because the required index {} is not in its database [{};{}].",
                        self.peer.address(),
                        *tangle.get_latest_solid_milestone_index() + 1,
                        packet.pruned_index,
                        packet.latest_solid_milestone_index
                    );
                    // TODO drop if autopeered.
                }

                // Also drop connection if autopeered and we can't help it sync

                self.peer.metrics().heartbeats_received_inc();
                self.metrics.heartbeats_received_inc();
            }
            _ => return Err(Error::UnsupportedPacketType(header.packet_type)),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
