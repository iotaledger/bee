// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::ProtocolConfig,
    event::MessageProcessed,
    helper,
    packet::Message as MessagePacket,
    peer::PeerManager,
    tangle::{MessageMetadata, MsTangle},
    worker::{
        message_submitter::MessageSubmitterError, BroadcasterWorker, BroadcasterWorkerEvent, IndexationPayloadWorker,
        IndexationPayloadWorkerEvent, MessageRequesterWorker, MetricsWorker, MilestonePayloadWorker,
        MilestonePayloadWorkerEvent, PeerManagerWorker, PropagatorWorker, PropagatorWorkerEvent, RequestedMessages,
        StorageWorker, TangleWorker,
    },
    ProtocolMetrics,
};

use bee_common::{event::Bus, packable::Packable, shutdown_stream::ShutdownStream};
use bee_common_pt2::{
    node::{Node, ResHandle},
    worker::Worker,
};
use bee_message::{payload::Payload, Message, MessageId};
use bee_network::PeerId;

use async_trait::async_trait;
use futures::{channel::oneshot::Sender, stream::StreamExt};
use log::{error, info, trace, warn};
use tokio::sync::mpsc;

use std::{any::TypeId, convert::Infallible};

pub(crate) struct ProcessorWorkerEvent {
    pub(crate) pow_score: f64,
    pub(crate) from: Option<PeerId>,
    pub(crate) message_packet: MessagePacket,
    pub(crate) notifier: Option<Sender<Result<MessageId, MessageSubmitterError>>>,
}

pub(crate) struct ProcessorWorker {
    pub(crate) tx: mpsc::UnboundedSender<ProcessorWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for ProcessorWorker {
    type Config = (ProtocolConfig, u64);
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<StorageWorker>(),
            TypeId::of::<TangleWorker>(),
            TypeId::of::<MilestonePayloadWorker>(),
            TypeId::of::<PropagatorWorker>(),
            TypeId::of::<BroadcasterWorker>(),
            TypeId::of::<MessageRequesterWorker>(),
            TypeId::of::<MetricsWorker>(),
            TypeId::of::<PeerManagerWorker>(),
            TypeId::of::<IndexationPayloadWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let milestone_payload_worker = node.worker::<MilestonePayloadWorker>().unwrap().tx.clone();
        let indexation_payload_worker = node.worker::<IndexationPayloadWorker>().unwrap().tx.clone();
        let propagator = node.worker::<PropagatorWorker>().unwrap().tx.clone();
        let broadcaster = node.worker::<BroadcasterWorker>().unwrap().tx.clone();
        let message_requester = node.worker::<MessageRequesterWorker>().unwrap().tx.clone();

        let _storage = node.storage();
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_messages = node.resource::<RequestedMessages>();
        let metrics = node.resource::<ProtocolMetrics>();
        let peer_manager = node.resource::<PeerManager>();
        let bus = node.resource::<Bus>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(ProcessorWorkerEvent {
                pow_score,
                from,
                message_packet,
                notifier,
            }) = receiver.next().await
            {
                trace!("Processing received message...");

                let message = match Message::unpack(&mut &message_packet.bytes[..]) {
                    Ok(message) => message,
                    Err(e) => {
                        invalid_message(format!("Invalid message: {:?}.", e), &metrics, notifier);
                        continue;
                    }
                };

                if message.network_id() != config.1 {
                    invalid_message(
                        format!("Incompatible network ID {} != {}.", message.network_id(), config.1),
                        &metrics,
                        notifier,
                    );
                    continue;
                }

                if pow_score < config.0.minimum_pow_score {
                    invalid_message(
                        format!(
                            "Insufficient pow score: {} < {}.",
                            pow_score, config.0.minimum_pow_score
                        ),
                        &metrics,
                        notifier,
                    );
                    continue;
                }

                // TODO should be passed by the hasher worker ?
                let (message_id, _) = message.id();
                let requested = requested_messages.contains_key(&message_id);

                let mut metadata = MessageMetadata::arrived();
                metadata.flags_mut().set_requested(requested);

                // store message
                if let Some(message) = tangle.insert(message, message_id, metadata).await {
                    bus.dispatch(MessageProcessed(message_id));

                    // TODO this was temporarily moved from the tangle.
                    // Reason is that since the tangle is not a worker, it can't have access to the propagator tx.
                    // When the tangle is made a worker, this should be put back on.

                    if let Err(e) = propagator.send(PropagatorWorkerEvent(message_id)) {
                        error!("Failed to send message id {} to propagator: {:?}.", message_id, e);
                    }

                    metrics.new_messages_inc();

                    match requested_messages.remove(&message_id) {
                        Some((_, (index, _))) => {
                            // Message was requested.
                            let parent1 = message.parent1();
                            let parent2 = message.parent2();

                            helper::request_message(&tangle, &message_requester, &*requested_messages, *parent1, index)
                                .await;
                            if parent1 != parent2 {
                                helper::request_message(
                                    &tangle,
                                    &message_requester,
                                    &*requested_messages,
                                    *parent2,
                                    index,
                                )
                                .await;
                            }
                        }
                        None => {
                            // Message was not requested.
                            if let Err(e) = broadcaster.send(BroadcasterWorkerEvent {
                                source: from,
                                message: message_packet,
                            }) {
                                warn!("Broadcasting message failed: {}.", e);
                            }
                        }
                    };

                    match message.payload() {
                        Some(Payload::Milestone(_)) => {
                            if let Err(e) = milestone_payload_worker.send(MilestonePayloadWorkerEvent(message_id)) {
                                error!(
                                    "Sending message id {} to milestone payload worker failed: {:?}.",
                                    message_id, e
                                );
                            }
                        }
                        Some(Payload::Indexation(_)) => {
                            if let Err(e) = indexation_payload_worker.send(IndexationPayloadWorkerEvent(message_id)) {
                                error!(
                                    "Sending message id {} to indexation payload worker failed: {:?}.",
                                    message_id, e
                                );
                            }
                        }
                        Some(Payload::Transaction(_)) => {
                            // TODO
                        }
                        Some(_) => {
                            // TODO
                        }
                        None => {
                            // TODO
                        }
                    }
                } else {
                    metrics.known_messages_inc();
                    if let Some(peer_id) = from {
                        peer_manager
                            .peers
                            .get(&peer_id)
                            .map(|peer| peer.metrics.known_messages_inc());
                    }
                }

                if let Some(notifier) = notifier {
                    if let Err(e) = notifier.send(Ok(message_id)) {
                        error!("Failed to send message id: {:?}.", e);
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}

fn invalid_message(
    error: String,
    metrics: &ResHandle<ProtocolMetrics>,
    notifier: Option<Sender<Result<MessageId, MessageSubmitterError>>>,
) {
    trace!("{}", error);
    metrics.invalid_messages_inc();

    if let Some(notifier) = notifier {
        if let Err(e) = notifier.send(Err(MessageSubmitterError(error))) {
            error!("Failed to send error: {:?}.", e);
        }
    }
}
