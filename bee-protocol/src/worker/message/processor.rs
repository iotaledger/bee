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
            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            use std::sync::atomic::{AtomicU64, Ordering};
            static TIME_TOTAL: AtomicU64 = AtomicU64::new(0);

            static ITER_TOTAL: AtomicU64 = AtomicU64::new(0);
            static TIME_UNPACK: AtomicU64 = AtomicU64::new(0);
            static TIME_NETWORKID: AtomicU64 = AtomicU64::new(0);
            static TIME_POWSCORE: AtomicU64 = AtomicU64::new(0);
            static TIME_REQUEST: AtomicU64 = AtomicU64::new(0);
            static TIME_INSERT: AtomicU64 = AtomicU64::new(0);
            static TIME_METRICS: AtomicU64 = AtomicU64::new(0);
            static TIME_BUS: AtomicU64 = AtomicU64::new(0);
            static TIME_PROPAGATE: AtomicU64 = AtomicU64::new(0);
            static TIME_REMOVEREQ: AtomicU64 = AtomicU64::new(0);
            static TIME_SEND: AtomicU64 = AtomicU64::new(0);
            const SAMPLE_ITERS: u64 = 250;

            while let Some(ProcessorWorkerEvent {
                pow_score,
                from,
                message_packet,
                notifier,
            }) = receiver.next().await
            {
                fn start() -> Instant { Instant::now() }
                fn end(i: Instant, total: &AtomicU64, s: &str) {
                    let t = total.fetch_add(i.elapsed().as_micros() as u64, Ordering::Relaxed);
                    if ITER_TOTAL.load(Ordering::Relaxed) == SAMPLE_ITERS {
                        println!("Avg time for {}: {} ({}% of total)", s, t as f32 / 1000.0, 100.0 * t as f32 / TIME_TOTAL.load(Ordering::Relaxed) as f32);
                        total.store(0, Ordering::Relaxed);
                    }
                }

                let now = std::time::Instant::now();
                trace!("Processing received message...");

                let s = start();
                let message = match Message::unpack(&mut &message_packet.bytes[..]) {
                    Ok(message) => message,
                    Err(e) => {
                        invalid_message(format!("Invalid message: {:?}.", e), &metrics, notifier);
                        continue;
                    }
                };
                end(s, &TIME_UNPACK, "unpack");

                let s = start();
                if message.network_id() != config.1 {
                    invalid_message(
                        format!("Incompatible network ID {} != {}.", message.network_id(), config.1),
                        &metrics,
                        notifier,
                    );
                    continue;
                }
                end(s, &TIME_NETWORKID, "network id");

                let s = start();
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
                end(s, &TIME_POWSCORE, "pow score");

                let s = start();
                // TODO should be passed by the hasher worker ?
                let (message_id, _) = message.id();
                let requested = requested_messages.contains(&message_id).await;

                let mut metadata = MessageMetadata::arrived();
                metadata.flags_mut().set_requested(requested);

                let parent1 = *message.parent1();
                let parent2 = *message.parent2();
                end(s, &TIME_REQUEST, "requested message");

                // store message
                let s = start();
                let inserted = tangle.insert(message, message_id, metadata).await.is_some();
                end(s, &TIME_INSERT, "insert");

                let s = start();
                if !inserted {
                    metrics.known_messages_inc();
                    if let Some(ref peer_id) = from {
                        peer_manager
                            .get(&peer_id)
                            .await
                            .map(|peer| (*peer).0.metrics().known_messages_inc());
                    }
                    continue;
                } else {
                    metrics.new_messages_inc();
                }
                end(s, &TIME_METRICS, "metrics");

                let s = start();
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
                end(s, &TIME_BUS, "bus");

                let s = start();
                if let Err(e) = propagator.send(PropagatorWorkerEvent(message_id)) {
                    error!("Failed to send message id {} to propagator: {:?}.", message_id, e);
                }
                end(s, &TIME_PROPAGATE, "propagate");

                let s = start();
                match requested_messages.remove(&message_id).await {
                    Some((index, instant)) => {
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
                end(s, &TIME_REMOVEREQ, "remove req");

                let s = start();
                if let Err(e) = payload_worker.send(PayloadWorkerEvent(message_id)) {
                    warn!("Sending message id {} to payload worker failed: {:?}.", message_id, e);
                } else {
                }

                if let Some(notifier) = notifier {
                    if let Err(e) = notifier.send(Ok(message_id)) {
                        error!("Failed to send message id: {:?}.", e);
                    }
                }
                end(s, &TIME_SEND, "send");

                let time = TIME_TOTAL.fetch_add(now.elapsed().as_micros() as u64, Ordering::Relaxed);
                let iter = ITER_TOTAL.fetch_add(1, Ordering::Relaxed);
                if iter == SAMPLE_ITERS {
                    println!("---- Processor body timings ----");
                    println!("Iterations = {}", iter);
                    println!("Time = {}us", time);
                    println!("Theoretical MPS: {}", iter as f32 / (time as f32 / 1000_000.0));

                    ITER_TOTAL.store(0, Ordering::Relaxed);
                    TIME_TOTAL.store(0, Ordering::Relaxed);
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
