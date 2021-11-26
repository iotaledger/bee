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

use bee_message::MessageId;
use bee_network::PeerId;
use bee_pow::score;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
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
            let mut pow = score::PoWScorer::new();

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

                let pow_score = pow.score(&message_packet.bytes);

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
