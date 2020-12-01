// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::assertions_on_constants)]

use crate::{
    packet::Message as MessagePacket,
    protocol::Protocol,
    worker::{
        message::{HashCache, ProcessorWorker, ProcessorWorkerEvent},
        message_submitter::MessageSubmitterError,
    },
};

use bee_common::{b1t6, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_crypto::ternary::{
    sponge::{BatchHasher, CurlPRounds, BATCH_SIZE},
    HASH_LENGTH,
};
use bee_message::{MessageId, MESSAGE_ID_LENGTH};
use bee_network::PeerId;
use bee_ternary::{Btrit, T5B1Buf, TritBuf};

use async_trait::async_trait;
use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use futures::{
    channel::oneshot::Sender,
    stream::{Fuse, Stream, StreamExt},
    task::{Context, Poll},
};
use log::{info, trace, warn};
use pin_project::pin_project;

use std::{any::TypeId, convert::Infallible, pin::Pin};

// If a batch has less than this number of messages, the regular CurlP hasher is used instead of the batched one.
const BATCH_SIZE_THRESHOLD: usize = 3;

pub(crate) struct HasherWorkerEvent {
    pub(crate) from: Option<PeerId>,
    pub(crate) message_packet: MessagePacket,
    pub(crate) notifier: Option<Sender<Result<MessageId, MessageSubmitterError>>>,
}

pub(crate) struct HasherWorker {
    pub(crate) tx: flume::Sender<HasherWorkerEvent>,
}

fn trigger_hashing(
    batch_size: usize,
    receiver: &mut BatchStream,
    processor_worker: &mut flume::Sender<ProcessorWorkerEvent>,
) {
    if batch_size < BATCH_SIZE_THRESHOLD {
        send_hashes(receiver.hasher.hash_unbatched(), &mut receiver.events, processor_worker);
    } else {
        send_hashes(receiver.hasher.hash_batched(), &mut receiver.events, processor_worker);
    }
    // FIXME: we could store the fraction of times we use the batched hasher
}

fn send_hashes(
    hashes: impl Iterator<Item = TritBuf>,
    events: &mut Vec<HasherWorkerEvent>,
    processor_worker: &mut flume::Sender<ProcessorWorkerEvent>,
) {
    for (
        HasherWorkerEvent {
            from,
            message_packet,
            notifier: message_inserted_tx,
        },
        hash,
    ) in events.drain(..).zip(hashes)
    {
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

#[async_trait]
impl<N: Node> Worker<N> for HasherWorker {
    type Config = usize;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<ProcessorWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();
        let mut processor_worker = node.worker::<ProcessorWorker>().unwrap().tx.clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            let mut receiver = BatchStream::new(config, ShutdownStream::new(shutdown, rx.into_stream()));

            info!("Running.");

            while let Some(batch_size) = receiver.next().await {
                trigger_hashing(batch_size, &mut receiver, &mut processor_worker);
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}

#[pin_project(project = BatchStreamProj)]
pub(crate) struct BatchStream {
    #[pin]
    receiver: ShutdownStream<Fuse<flume::r#async::RecvStream<'static, HasherWorkerEvent>>>,
    cache: HashCache,
    hasher: BatchHasher<T5B1Buf>,
    events: Vec<HasherWorkerEvent>,
    blake2b: VarBlake2b,
}

impl BatchStream {
    pub(crate) fn new(
        cache_size: usize,
        receiver: ShutdownStream<Fuse<flume::r#async::RecvStream<'static, HasherWorkerEvent>>>,
    ) -> Self {
        assert!(BATCH_SIZE_THRESHOLD <= BATCH_SIZE);
        Self {
            receiver,
            cache: HashCache::new(cache_size),
            hasher: BatchHasher::new(HASH_LENGTH, CurlPRounds::Rounds81),
            events: Vec::with_capacity(BATCH_SIZE),
            blake2b: VarBlake2b::new(MESSAGE_ID_LENGTH).unwrap(),
        }
    }
}

impl Stream for BatchStream {
    type Item = usize;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        // We need to do this because `receiver` needs to be pinned to be polled.
        let BatchStreamProj {
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
                return Poll::Ready(Some(BATCH_SIZE));
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
                        Poll::Ready(Some(batch_size))
                    };
                }
                Poll::Ready(Some(event)) => {
                    // If the message was already received, we skip it and poll again.
                    if !cache.insert(&event.message_packet.bytes) {
                        trace!("Message already received.");
                        Protocol::get().metrics.known_messages_inc();
                        continue;
                    }

                    // Given that the current batch has less than `BATCH_SIZE` messages, we can add the message in
                    // the current event to the batch.

                    // TODO const
                    // TODO check
                    blake2b.update(&event.message_packet.bytes[..event.message_packet.bytes.len() - 8]);
                    let mut pow_input = TritBuf::with_capacity(243);
                    blake2b.finalize_variable_reset(|pow_digest| {
                        b1t6::encode(&pow_digest).iter().for_each(|t| pow_input.push(t))
                    });
                    b1t6::encode(&event.message_packet.bytes[event.message_packet.bytes.len() - 8..])
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
                        return Poll::Ready(Some(BATCH_SIZE));
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
