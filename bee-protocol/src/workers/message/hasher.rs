// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::assertions_on_constants)]

use crate::{
    types::metrics::NodeMetrics,
    workers::{
        config::ProtocolConfig,
        message::{
            submitter::notify_invalid_message, HashCache, MessageSubmitterError, ProcessorWorker, ProcessorWorkerEvent,
        },
        packets::MessagePacket,
        storage::StorageBackend,
        MetricsWorker, PeerManager, PeerManagerResWorker,
    },
};

use bee_crypto::ternary::sponge::{Sponge, UnrolledCurlP81};
use bee_message::MessageId;
use bee_network::PeerId;
use bee_pow::score;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_ternary::{b1t6, Btrit, T1B1Buf, TritBuf};

use async_trait::async_trait;
use crypto::hashes::{blake2b::Blake2b256, Digest};
use futures::{channel::oneshot::Sender, StreamExt};
use log::{error, info, trace, warn};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, convert::Infallible};

pub(crate) struct HasherWorkerEvent {
    pub(crate) from: Option<PeerId>,
    pub(crate) message_packet: MessagePacket,
    pub(crate) notifier: Option<Sender<Result<MessageId, MessageSubmitterError>>>,
}

pub(crate) struct HasherWorker {
    pub(crate) tx: mpsc::UnboundedSender<HasherWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for HasherWorker
where
    N::Backend: StorageBackend,
{
    type Config = ProtocolConfig;
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

        let minimum_pow_score = config.minimum_pow_score;

        let mut cache = HashCache::new(config.workers.message_worker_cache);

        node.spawn::<Self, _, _>(|shutdown| async move {
            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));
            let mut hasher = UnrolledCurlP81::new();
            let mut blake2b = Blake2b256::new();

            info!("Running.");

            while let Some(HasherWorkerEvent {
                from,
                message_packet,
                notifier,
            }) = receiver.next().await
            {
                if !cache.insert(&message_packet.bytes) {
                    // If the message was already received, we skip it and poll again.
                    trace!("Message already received.");

                    if let Some(notifier) = notifier {
                        if let Err(e) =
                            notifier.send(Err(MessageSubmitterError("message already received".to_string())))
                        {
                            error!("failed to send error: {:?}.", e);
                        }
                    }

                    metrics.known_messages_inc();
                    if let Some(peer_id) = from {
                        if let Some(peer) = peer_manager.get(&peer_id).await {
                            (*peer).0.metrics().known_messages_inc();
                        }
                    }
                    continue;
                }

                // TODO const
                // TODO check
                // TODO see if there is something we can reuse from pow crate
                let (head, tail) = message_packet.bytes.split_at(message_packet.bytes.len() - 8);
                blake2b.update(head);

                let mut pow_input: TritBuf = TritBuf::with_capacity(243);
                let pow_digest = blake2b.finalize_reset();

                b1t6::encode::<T1B1Buf>(&pow_digest)
                    .iter()
                    .for_each(|t| pow_input.push(t));
                b1t6::encode::<T1B1Buf>(tail).iter().for_each(|t| pow_input.push(t));

                // Pad to 243 trits.
                pow_input.push(Btrit::Zero);
                pow_input.push(Btrit::Zero);
                pow_input.push(Btrit::Zero);

                let hash = hasher.digest(&pow_input).unwrap();

                let pow_score = score::pow_score(&hash, message_packet.bytes.len());

                if pow_score < minimum_pow_score {
                    notify_invalid_message(
                        format!("Insufficient pow score: {} < {}.", pow_score, minimum_pow_score),
                        &metrics,
                        notifier,
                    );
                    continue;
                }

                if let Err(e) = processor_worker.send(ProcessorWorkerEvent {
                    from,
                    message_packet,
                    notifier,
                }) {
                    warn!("Sending event to the processor worker failed: {}.", e);
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
