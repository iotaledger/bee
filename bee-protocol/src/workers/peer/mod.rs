// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod manager;
mod manager_res;
mod packet_handler;

pub(crate) use manager::PeerManagerWorker;
pub use manager_res::{PeerManager, PeerManagerResWorker};

use crate::{
    types::{metrics::NodeMetrics, peer::Peer},
    workers::{
        packets::{
            tlv_from_bytes, HeaderPacket, HeartbeatPacket, MessagePacket, MessageRequestPacket, MilestoneRequestPacket,
            Packet, TlvError,
        },
        peer::packet_handler::PacketHandler,
        requester::request_latest_milestone,
        storage::StorageBackend,
        HasherWorkerEvent, MessageResponderWorkerEvent, MilestoneRequesterWorkerEvent, MilestoneResponderWorkerEvent,
        RequestedMilestones,
    },
};

use bee_message::milestone::MilestoneIndex;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use futures::{channel::oneshot, future::FutureExt};
use log::{debug, error, info, trace};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::sync::Arc;

#[derive(Debug)]
pub(crate) enum Error {
    UnsupportedPacketType(u8),
    TlvError(TlvError),
}

impl From<TlvError> for Error {
    fn from(error: TlvError) -> Self {
        Error::TlvError(error)
    }
}

pub struct PeerWorker {
    peer: Arc<Peer>,
    metrics: ResourceHandle<NodeMetrics>,
    hasher: mpsc::UnboundedSender<HasherWorkerEvent>,
    message_responder: mpsc::UnboundedSender<MessageResponderWorkerEvent>,
    milestone_responder: mpsc::UnboundedSender<MilestoneResponderWorkerEvent>,
    milestone_requester: mpsc::UnboundedSender<MilestoneRequesterWorkerEvent>,
}

impl PeerWorker {
    pub(crate) fn new(
        peer: Arc<Peer>,
        metrics: ResourceHandle<NodeMetrics>,
        hasher: mpsc::UnboundedSender<HasherWorkerEvent>,
        message_responder: mpsc::UnboundedSender<MessageResponderWorkerEvent>,
        milestone_responder: mpsc::UnboundedSender<MilestoneResponderWorkerEvent>,
        milestone_requester: mpsc::UnboundedSender<MilestoneRequesterWorkerEvent>,
    ) -> Self {
        Self {
            peer,
            metrics,
            hasher,
            message_responder,
            milestone_responder,
            milestone_requester,
        }
    }

    pub(crate) async fn run<B: StorageBackend>(
        mut self,
        tangle: ResourceHandle<MsTangle<B>>,
        requested_milestones: ResourceHandle<RequestedMilestones>,
        receiver: UnboundedReceiverStream<Vec<u8>>,
        shutdown: oneshot::Receiver<()>,
    ) {
        info!("[{}] Running.", self.peer.alias());

        let shutdown_fused = shutdown.fuse();

        let mut packet_handler = PacketHandler::new(receiver, shutdown_fused, self.peer.address().clone());

        request_latest_milestone(
            &*tangle,
            &self.milestone_requester,
            &*requested_milestones,
            Some(*self.peer.id()),
        )
        .await;

        // TODO is this needed ?
        let tangle = tangle.into_weak();

        while let Some((header, bytes)) = packet_handler.fetch_packet().await {
            let tangle = tangle.upgrade().expect("Needed Tangle resource but it was removed");

            if let Err(e) = self.process_packet(&tangle, &header, bytes) {
                error!("[{}] Processing packet failed: {:?}.", self.peer.alias(), e);
                self.peer.metrics().invalid_packets_inc();
                self.metrics.invalid_packets_inc();
            }
        }

        info!("[{}] Stopped.", self.peer.alias());
    }

    fn process_packet<B: StorageBackend>(
        &mut self,
        tangle: &MsTangle<B>,
        header: &HeaderPacket,
        bytes: &[u8],
    ) -> Result<(), Error> {
        match header.packet_type {
            MilestoneRequestPacket::ID => {
                trace!("[{}] Reading MilestoneRequestPacket...", self.peer.alias());

                let packet = tlv_from_bytes::<MilestoneRequestPacket>(&header, bytes)?;

                let _ = self.milestone_responder.send(MilestoneResponderWorkerEvent {
                    peer_id: *self.peer.id(),
                    request: packet,
                });

                self.peer.metrics().milestone_requests_received_inc();
                self.metrics.milestone_requests_received_inc();
            }
            MessagePacket::ID => {
                trace!("[{}] Reading MessagePacket...", self.peer.alias());

                let packet = tlv_from_bytes::<MessagePacket>(&header, bytes)?;

                let _ = self.hasher.send(HasherWorkerEvent {
                    from: Some(*self.peer.id()),
                    message_packet: packet,
                    notifier: None,
                });

                self.peer.metrics().messages_received_inc();
                self.metrics.messages_received_inc();
            }
            MessageRequestPacket::ID => {
                trace!("[{}] Reading MessageRequestPacket...", self.peer.alias());

                let packet = tlv_from_bytes::<MessageRequestPacket>(&header, bytes)?;

                let _ = self.message_responder.send(MessageResponderWorkerEvent {
                    peer_id: *self.peer.id(),
                    request: packet,
                });

                self.peer.metrics().message_requests_received_inc();
                self.metrics.message_requests_received_inc();
            }
            HeartbeatPacket::ID => {
                trace!("[{}] Reading HeartbeatPacket...", self.peer.alias());

                let packet = tlv_from_bytes::<HeartbeatPacket>(&header, bytes)?;

                self.peer.set_solid_milestone_index(packet.solid_milestone_index.into());
                self.peer.set_pruned_index(packet.pruned_index.into());
                self.peer
                    .set_latest_milestone_index(packet.latest_milestone_index.into());
                self.peer.set_connected_peers(packet.connected_peers);
                self.peer.set_synced_peers(packet.synced_peers);
                self.peer.set_heartbeat_received_timestamp();

                if !tangle.is_synced()
                    && !self
                        .peer
                        .has_data(MilestoneIndex(*tangle.get_solid_milestone_index() + 1))
                {
                    debug!(
                        "The peer {} can't help syncing because the required index {} is not in its database [{};{}].",
                        self.peer.alias(),
                        *tangle.get_solid_milestone_index() + 1,
                        packet.pruned_index,
                        packet.solid_milestone_index
                    );
                }

                self.peer.metrics().heartbeats_received_inc();
                self.metrics.heartbeats_received_inc();
            }
            _ => return Err(Error::UnsupportedPacketType(header.packet_type)),
        };

        Ok(())
    }
}
