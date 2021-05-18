// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::assertions_on_constants)]

use crate::{
    types::metrics::NodeMetrics,
    workers::{
        message::{HashCache, MessageSubmitterError, ProcessorWorker, ProcessorWorkerEvent},
        packets::MessagePacket,
        peer::PeerManager,
        storage::StorageBackend,
        MetricsWorker, PeerManagerResWorker,
    },
};

use bee_crypto::ternary::{
    sponge::{BatchHasher, CurlPRounds, BATCH_SIZE},
    HASH_LENGTH,
};
use bee_message::MessageId;
use bee_network::PeerId;
use bee_runtime::{node::Node, resource::ResourceHandle, shutdown_stream::ShutdownStream, worker::Worker};
use bee_ternary::{b1t6, Btrit, T1B1Buf, T5B1Buf, TritBuf};

use async_trait::async_trait;
use crypto::hashes::{blake2b::Blake2b256, Digest};
use futures::{
    channel::oneshot::Sender,
    stream::{unfold, Fuse},
    task::{Context, Poll},
    FutureExt, Stream, StreamExt,
};
use log::{info, trace, warn};
use pin_project::pin_project;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, convert::Infallible, pin::Pin};

// If a batch has less than this number of messages, the regular CurlP hasher is used instead of the batched one.
const BATCH_SIZE_THRESHOLD: usize = 3;

pub(crate) struct HasherWorkerEvent {
    pub(crate) from: Option<PeerId>,
    pub(crate) message_packet: MessagePacket,
    pub(crate) notifier: Option<Sender<Result<MessageId, MessageSubmitterError>>>,
}

pub(crate) struct HasherWorker {
    pub(crate) tx: mpsc::UnboundedSender<HasherWorkerEvent>,
}

// fn trigger_hashing(
//     batch_size: usize,
//     receiver: &mut BatchStream,
//     processor_worker: &mut mpsc::UnboundedSender<ProcessorWorkerEvent>,
// ) {
//     if batch_size < BATCH_SIZE_THRESHOLD {
//         send_hashes(receiver.hasher.hash_unbatched(), &mut receiver.events, processor_worker);
//     } else {
//         send_hashes(receiver.hasher.hash_batched(), &mut receiver.events, processor_worker);
//     }
//     // FIXME: we could store the fraction of times we use the batched hasher
// }

fn send_hashes(
    hashes: impl Iterator<Item = TritBuf>,
    events: Vec<HasherWorkerEvent>,
    processor_worker: &mut mpsc::UnboundedSender<ProcessorWorkerEvent>,
) {
    for (
        HasherWorkerEvent {
            from,
            message_packet,
            notifier: message_inserted_tx,
        },
        hash,
    ) in events.into_iter().zip(hashes)
    {
        // TODO replace this with scoring function
        let zeros = hash.iter().rev().take_while(|t| *t == Btrit::Zero).count() as u32;
        let pow_score = 3u128.pow(zeros) as f64 / message_packet.bytes.len() as f64;
        // TODO check score

        if let Err(e) = processor_worker.send(ProcessorWorkerEvent {
            pow_score,
            from,
            message_packet,
            notifier: message_inserted_tx,
        }) {
            warn!("Sending event to the processor worker failed: {}.", e);
        }
    }
}

pub(crate) struct HashTask {
    batch_size: usize,
    hasher: BatchHasher<T5B1Buf>,
    events: Vec<HasherWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for HasherWorker
where
    N::Backend: StorageBackend,
{
    type Config = usize;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<ProcessorWorker>(),
            TypeId::of::<MetricsWorker>(),
            TypeId::of::<PeerManagerResWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let processor_worker = node.worker::<ProcessorWorker>().unwrap().tx.clone();
        let metrics = node.resource::<NodeMetrics>();
        let peer_manager = node.resource::<PeerManager>();

        let hash_tasks = num_cpus::get();
        let (task_tx, task_rx) = async_channel::unbounded();

        for _ in 0..hash_tasks {
            let task_rx = task_rx.clone();
            let mut processor_worker = processor_worker.clone();
            node.spawn::<Self, _, _>(|shutdown| async move {
                let mut s = ShutdownStream::new(
                    shutdown,
                    unfold((), |()| task_rx.recv().map(|t| Some((t.ok()?, ())))).boxed(),
                );

                while let Some(HashTask {
                    batch_size,
                    mut hasher,
                    events,
                }) = s.next().await
                {
                    tokio::task::block_in_place(|| {
                        if batch_size < BATCH_SIZE_THRESHOLD {
                            send_hashes(hasher.hash_unbatched(), events, &mut processor_worker);
                        } else {
                            send_hashes(hasher.hash_batched(), events, &mut processor_worker);
                        }
                    });
                }
            });
        }

        node.spawn::<Self, _, _>(|shutdown| async move {
            let mut receiver = BatchStream::new(
                config,
                metrics,
                peer_manager,
                ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx)),
            );

            info!("Running.");

            while let Some(task) = receiver.next().await {
                let _ = task_tx.send(task).await;
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}

#[pin_project(project = BatchStreamProj)]
pub(crate) struct BatchStream {
    metrics: ResourceHandle<NodeMetrics>,
    peer_manager: ResourceHandle<PeerManager>,
    #[pin]
    receiver: ShutdownStream<Fuse<UnboundedReceiverStream<HasherWorkerEvent>>>,
    cache: HashCache,
    hasher: BatchHasher<T5B1Buf>,
    events: Vec<HasherWorkerEvent>,
    blake2b: Blake2b256,
}

impl BatchStream {
    pub(crate) fn new(
        cache_size: usize,
        metrics: ResourceHandle<NodeMetrics>,
        peer_manager: ResourceHandle<PeerManager>,
        receiver: ShutdownStream<Fuse<UnboundedReceiverStream<HasherWorkerEvent>>>,
    ) -> Self {
        assert!(BATCH_SIZE_THRESHOLD <= BATCH_SIZE);
        Self {
            metrics,
            peer_manager,
            receiver,
            cache: HashCache::new(cache_size),
            hasher: BatchHasher::new(HASH_LENGTH, CurlPRounds::Rounds81),
            events: Vec::with_capacity(BATCH_SIZE),
            blake2b: Blake2b256::new(),
        }
    }
}

impl Stream for BatchStream {
    type Item = HashTask;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        // We need to do this because `receiver` needs to be pinned to be polled.
        let BatchStreamProj {
            metrics: _metrics,
            peer_manager: _peer_manager,
            mut receiver,
            cache,
            hasher,
            events,
            blake2b,
            ..
        } = self.project();

        // We loop until we have `BATCH_SIZE` messages or `stream.poll_next(cx)` returns pending.
        loop {
            let batch_size = hasher.len();
            // If we already have `BATCH_SIZE` messages, we are ready.
            if batch_size == BATCH_SIZE {
                return Poll::Ready(Some(HashTask {
                    batch_size: BATCH_SIZE,
                    hasher: std::mem::replace(hasher, BatchHasher::new(HASH_LENGTH, CurlPRounds::Rounds81)),
                    events: std::mem::take(events),
                }));
            }
            // Otherwise we need to check if there are messages inside the `receiver` stream that we could include in
            // the current batch.
            match receiver.as_mut().poll_next(cx) {
                Poll::Pending => {
                    return if batch_size == 0 {
                        // If the stream is not ready yet and the current batch is empty we have to wait.
                        // Otherwise, we would end up hashing an empty batch.
                        Poll::Pending
                    } else {
                        // If the stream is not ready yet, but we have messages in the batch, we can process them
                        // instead of waiting.
                        Poll::Ready(Some(HashTask {
                            batch_size,
                            hasher: std::mem::replace(hasher, BatchHasher::new(HASH_LENGTH, CurlPRounds::Rounds81)),
                            events: std::mem::take(events),
                        }))
                    };
                }
                Poll::Ready(Some(event)) => {
                    // If the message was already received, we skip it and poll again.
                    if !cache.insert(&event.message_packet.bytes) {
                        trace!("Message already received.");
                        // TODO put it back
                        // metrics.known_messages_inc();
                        // if let Some(peer_id) = event.from {
                        //     if let Some(peer) = peer_manager.get(&peer_id).await {
                        //         peer.value().0.metrics().known_messages_inc();
                        //     }
                        // }
                        continue;
                    }

                    // Given that the current batch has less than `BATCH_SIZE` messages, we can add the message in
                    // the current event to the batch.

                    // TODO const
                    // TODO check
                    // TODO see if there is something we can reuse from pow crate
                    blake2b.update(&event.message_packet.bytes[..event.message_packet.bytes.len() - 8]);

                    let mut pow_input = TritBuf::with_capacity(243);
                    let pow_digest = blake2b.finalize_reset();

                    b1t6::encode::<T1B1Buf>(&pow_digest)
                        .iter()
                        .for_each(|t| pow_input.push(t));
                    b1t6::encode::<T1B1Buf>(&event.message_packet.bytes[event.message_packet.bytes.len() - 8..])
                        .iter()
                        .for_each(|t| pow_input.push(t));
                    // Pad to 243 trits.
                    pow_input.push(Btrit::Zero);
                    pow_input.push(Btrit::Zero);
                    pow_input.push(Btrit::Zero);

                    // TODO check that size == 243

                    hasher.add(pow_input);
                    events.push(event);

                    // If after adding the message to the batch its size is `BATCH_SIZE` we are ready to hash.
                    if batch_size == BATCH_SIZE - 1 {
                        return Poll::Ready(Some(HashTask {
                            batch_size: BATCH_SIZE,
                            hasher: std::mem::replace(hasher, BatchHasher::new(HASH_LENGTH, CurlPRounds::Rounds81)),
                            events: std::mem::take(events),
                        }));
                    }
                }
                Poll::Ready(None) => {
                    // If the `receiver` stream ended, it means that either we should shutdown or
                    // the other side of the channel disconnected. In either case, we end this
                    // stream too without hashing the pending batch we have.
                    return Poll::Ready(None);
                }
            }
        }
    }
}
