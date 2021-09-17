// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::metrics::NodeMetrics,
    workers::{
        event::{MessageProcessed, VertexCreated},
        message::submitter::{notify_invalid_message, notify_message},
        packets::MessagePacket,
        peer::PeerManager,
        requester::request_message,
        storage::StorageBackend,
        BroadcasterWorker, BroadcasterWorkerEvent, MessageRequesterWorker, MessageSubmitterError, MetricsWorker,
        PayloadWorker, PayloadWorkerEvent, PeerManagerResWorker, PropagatorWorker, PropagatorWorkerEvent,
        RequestedMessages, UnreferencedMessageInserterWorker, UnreferencedMessageInserterWorkerEvent,
    },
};

use bee_common::packable::Packable;
use bee_message::{Message, MessageId};
use bee_network::PeerId;
use bee_runtime::{
    node::Node,
    shutdown_stream::ShutdownStream,
    task::{StandaloneSpawner, TaskSpawner},
    worker::Worker,
};
use bee_tangle::{metadata::MessageMetadata, MsTangle, TangleWorker};

use async_trait::async_trait;
use futures::{channel::oneshot::Sender, stream::StreamExt};
use log::{error, info, trace};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, convert::Infallible, time::Instant};

pub(crate) struct ProcessorWorkerEvent {
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
    type Config = u64;
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
            TypeId::of::<UnreferencedMessageInserterWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let propagator = node.worker::<PropagatorWorker>().unwrap().tx.clone();
        let broadcaster = node.worker::<BroadcasterWorker>().unwrap().tx.clone();
        let message_requester = node.worker::<MessageRequesterWorker>().unwrap().clone();
        let payload_worker = node.worker::<PayloadWorker>().unwrap().tx.clone();
        let unreferenced_inserted_worker = node.worker::<UnreferencedMessageInserterWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_messages = node.resource::<RequestedMessages>();
        let metrics = node.resource::<NodeMetrics>();
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
                let unreferenced_inserted_worker = unreferenced_inserted_worker.clone();
                let tangle = tangle.clone();
                let requested_messages = requested_messages.clone();
                let metrics = metrics.clone();
                let peer_manager = peer_manager.clone();
                let bus = bus.clone();
                let network_id = config;

                StandaloneSpawner::spawn(async move {
                    while let Ok(ProcessorWorkerEvent {
                        from,
                        message_packet,
                        notifier,
                    }) = rx.recv().await
                    {
                        trace!("Processing received message...");

                        let message = match Message::unpack(&mut &message_packet.bytes[..]) {
                            Ok(message) => message,
                            Err(e) => {
                                notify_invalid_message(format!("Invalid message: {:?}.", e), &metrics, notifier);
                                continue;
                            }
                        };

                        if message.network_id() != network_id {
                            notify_invalid_message(
                                format!("Incompatible network ID {} != {}.", message.network_id(), network_id),
                                &metrics,
                                notifier,
                            );
                            continue;
                        }

                        let (message_id, _) = message.id();
                        let metadata = MessageMetadata::arrived();

                        let message = if let Some(message) = tangle.insert(message, message_id, metadata).await {
                            message
                        } else {
                            metrics.known_messages_inc();
                            if let Some(ref peer_id) = from {
                                peer_manager
                                    .get(&peer_id)
                                    .await
                                    .map(|peer| (*peer).0.metrics().known_messages_inc());
                            }
                            continue;
                        };

                        // Send the propagation event ASAP to allow the propagator to do its thing
                        if let Err(e) = propagator.send(PropagatorWorkerEvent(message_id)) {
                            error!("Failed to send message id {} to propagator: {:?}.", message_id, e);
                        }

                        match requested_messages.remove(&message_id).await {
                            Some((index, instant)) => {
                                // Message was requested.

                                latency_num += 1;
                                latency_sum += (Instant::now() - instant).as_millis() as u64;
                                metrics.messages_average_latency_set(latency_sum / latency_num);

                                for parent in message.parents().iter() {
                                    request_message(&tangle, &message_requester, &*requested_messages, *parent, index)
                                        .await;
                                }
                            }
                            None => {
                                // Message was not requested.
                                if let Err(e) = broadcaster.send(BroadcasterWorkerEvent {
                                    source: from,
                                    message: message_packet,
                                }) {
                                    error!("Broadcasting message failed: {}.", e);
                                }
                                if let Err(e) =
                                    unreferenced_inserted_worker.send(UnreferencedMessageInserterWorkerEvent(
                                        message_id,
                                        tangle.get_latest_milestone_index(),
                                    ))
                                {
                                    error!("Sending message to unreferenced inserter failed: {}.", e);
                                }
                            }
                        };

                        if payload_worker
                            .send(PayloadWorkerEvent {
                                message_id,
                                message: message.clone(),
                            })
                            .is_err()
                        {
                            error!("Sending message {} to payload worker failed.", message_id);
                        }

                        notify_message(message_id, notifier);

                        bus.dispatch(MessageProcessed { message_id });

                        // TODO: boolean values are false at this point in time? trigger event from another location?
                        bus.dispatch(VertexCreated {
                            message_id,
                            parent_message_ids: message.parents().to_vec(),
                            is_solid: false,
                            is_referenced: false,
                            is_conflicting: false,
                            is_milestone: false,
                            is_tip: false,
                            is_selected: false,
                        });

                        metrics.new_messages_inc();
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
