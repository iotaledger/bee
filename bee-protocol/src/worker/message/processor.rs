// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::ProtocolConfig,
    event::{MessageProcessed, NewVertex},
    helper,
    packet::Message as MessagePacket,
    peer::PeerManager,
    storage::StorageBackend,
    worker::{
        BroadcasterWorker, BroadcasterWorkerEvent, MessageRequesterWorker, MessageSubmitterError, MetricsWorker,
        PayloadWorker, PayloadWorkerEvent, PeerManagerResWorker, PropagatorWorker, PropagatorWorkerEvent,
        RequestedMessages, TangleWorker,
    },
    ProtocolMetrics,
};

use bee_common::packable::Packable;
use bee_message::{Message, MessageId};
use bee_network::PeerId;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{metadata::MessageMetadata, MsTangle};

use async_trait::async_trait;
use futures::{channel::oneshot::Sender, stream::StreamExt};
use log::{error, info, trace, warn};
use tokio::sync::mpsc;

use std::{any::TypeId, convert::Infallible, time::Instant};

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
impl<N: Node> Worker<N> for ProcessorWorker
where
    N::Backend: StorageBackend,
{
    type Config = (ProtocolConfig, u64);
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<PropagatorWorker>(),
            TypeId::of::<BroadcasterWorker>(),
            TypeId::of::<MessageRequesterWorker>(),
            TypeId::of::<MetricsWorker>(),
            TypeId::of::<PeerManagerResWorker>(),
            TypeId::of::<PayloadWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let propagator = node.worker::<PropagatorWorker>().unwrap().tx.clone();
        let broadcaster = node.worker::<BroadcasterWorker>().unwrap().tx.clone();
        let message_requester = node.worker::<MessageRequesterWorker>().unwrap().tx.clone();
        let payload_worker = node.worker::<PayloadWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_messages = node.resource::<RequestedMessages>();
        let metrics = node.resource::<ProtocolMetrics>();
        let peer_manager = node.resource::<PeerManager>();
        let bus = node.bus();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut latency_num: u64 = 0;
            let mut latency_sum: u64 = 0;
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

                let parent1 = *message.parent1();
                let parent2 = *message.parent2();

                // store message
                let inserted = tangle.insert(message, message_id, metadata).await.is_some();

                if !inserted {
                    metrics.known_messages_inc();
                    if let Some(ref peer_id) = from {
                        peer_manager
                            .get(&peer_id)
                            .map(|peer| peer.value().0.metrics().known_messages_inc());
                    }
                    continue;
                }

                bus.dispatch(MessageProcessed(message_id));

                // TODO: boolean values are false at this point in time? trigger event from another location?
                bus.dispatch(NewVertex {
                    id: message_id.to_string(),
                    parent1_id: parent1.to_string(),
                    parent2_id: parent2.to_string(),
                    is_solid: false,
                    is_referenced: false,
                    is_conflicting: false,
                    is_milestone: false,
                    is_tip: false,
                    is_selected: false,
                });

                if let Err(e) = propagator.send(PropagatorWorkerEvent(message_id)) {
                    error!("Failed to send message id {} to propagator: {:?}.", message_id, e);
                }

                metrics.new_messages_inc();

                match requested_messages.remove(&message_id) {
                    Some((_, (index, instant))) => {
                        // Message was requested.

                        latency_num += 1;
                        latency_sum += (Instant::now() - instant).as_millis() as u64;
                        metrics.messages_average_latency_set(latency_sum / latency_num);

                        helper::request_message(&tangle, &message_requester, &*requested_messages, parent1, index)
                            .await;
                        if parent1 != parent2 {
                            helper::request_message(&tangle, &message_requester, &*requested_messages, parent2, index)
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

                if let Err(e) = payload_worker.send(PayloadWorkerEvent(message_id)) {
                    warn!("Sending message id {} to payload worker failed: {:?}.", message_id, e);
                } else {
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
    metrics: &ProtocolMetrics,
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
