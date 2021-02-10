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
        RequestedMessages, UnconfirmedMessageInserterWorker, UnconfirmedMessageInserterWorkerEvent,
    },
    ProtocolMetrics,
};

use bee_common::packable::Packable;
use bee_message::{Message, MessageId};
use bee_network::PeerId;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{metadata::MessageMetadata, MsTangle, TangleWorker};

use async_trait::async_trait;
use futures::{channel::oneshot::Sender, stream::StreamExt};
use log::{error, info, trace, warn};
use tokio::{sync::mpsc, task};
use tokio_stream::wrappers::UnboundedReceiverStream;

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
            TypeId::of::<UnconfirmedMessageInserterWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let propagator = node.worker::<PropagatorWorker>().unwrap().tx.clone();
        let broadcaster = node.worker::<BroadcasterWorker>().unwrap().tx.clone();
        let message_requester = node.worker::<MessageRequesterWorker>().unwrap().clone();
        let payload_worker = node.worker::<PayloadWorker>().unwrap().tx.clone();
        let unconfirmed_inserted_worker = node.worker::<UnconfirmedMessageInserterWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_messages = node.resource::<RequestedMessages>();
        let metrics = node.resource::<ProtocolMetrics>();
        let peer_manager = node.resource::<PeerManager>();
        let bus = node.bus();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut latency_num: u64 = 0;
            let mut latency_sum: u64 = 0;
            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            let (tx, rx) = async_channel::unbounded();

            for _ in 0..16 {
                let rx = rx.clone();
                let propagator = propagator.clone();
                let broadcaster = broadcaster.clone();
                let message_requester = message_requester.clone();
                let payload_worker = payload_worker.clone();
                let unconfirmed_inserted_worker = unconfirmed_inserted_worker.clone();
                let tangle = tangle.clone();
                let requested_messages = requested_messages.clone();
                let metrics = metrics.clone();
                let peer_manager = peer_manager.clone();
                let bus = bus.clone();
                let config = config.clone();

                task::spawn(async move {
                    while let Ok(ProcessorWorkerEvent {
                        pow_score,
                        from,
                        message_packet,
                        notifier,
                    }) = rx.recv().await
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

                        let metadata = MessageMetadata::arrived();

                        let parents = message.parents().to_vec();

                        // store message
                        let inserted = tangle.insert(message, message_id, metadata).await.is_some();

                        if !inserted {
                            metrics.known_messages_inc();
                            if let Some(ref peer_id) = from {
                                peer_manager
                                    .get(&peer_id)
                                    .await
                                    .map(|peer| (*peer).0.metrics().known_messages_inc());
                            }
                            continue;
                        }

                        bus.dispatch(MessageProcessed(message_id));

                        // TODO: boolean values are false at this point in time? trigger event from another location?
                        bus.dispatch(NewVertex {
                            id: message_id.to_string(),
                            parent_ids: parents.iter().map(|p| p.to_string()).collect(),
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

                        match requested_messages.remove(&message_id).await {
                            Some((index, instant)) => {
                                // Message was requested.

                                latency_num += 1;
                                latency_sum += (Instant::now() - instant).as_millis() as u64;
                                metrics.messages_average_latency_set(latency_sum / latency_num);

                                for parent in parents.iter() {
                                    // TODO only request once if same parents
                                    helper::request_message(
                                        &tangle,
                                        &message_requester,
                                        &*requested_messages,
                                        *parent,
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
                                if let Err(e) = unconfirmed_inserted_worker.send(UnconfirmedMessageInserterWorkerEvent(
                                    message_id,
                                    tangle.get_latest_milestone_index(),
                                )) {
                                    warn!("Sending message to unconfirmed inserter failed: {}.", e);
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
                });
            }

            while let Some(event) = receiver.next().await {
                let _ = tx.send(event).await;
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
