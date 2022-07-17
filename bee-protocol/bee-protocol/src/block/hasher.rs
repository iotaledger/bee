// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, convert::Infallible};

use async_trait::async_trait;
use bee_block::BlockId;
use bee_gossip::PeerId;
use bee_pow::score;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use futures::{channel::oneshot::Sender, StreamExt};
use log::{error, info, trace, warn};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    types::metrics::NodeMetrics,
    {
        block::{BlockSubmitterError, HashCache, ProcessorWorker, ProcessorWorkerEvent},
        config::ProtocolConfig,
        packets::BlockPacket,
        storage::StorageBackend,
        MetricsWorker, PeerManager, PeerManagerResWorker,
    },
};

pub(crate) struct HasherWorkerEvent {
    pub(crate) from: Option<PeerId>,
    pub(crate) block_packet: BlockPacket,
    pub(crate) notifier: Option<Sender<Result<BlockId, BlockSubmitterError>>>,
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

        let mut cache = HashCache::new(config.workers.block_worker_cache);

        node.spawn::<Self, _, _>(|shutdown| async move {
            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));
            let mut pow = score::PoWScorer::new();

            info!("Running.");

            while let Some(HasherWorkerEvent {
                from,
                block_packet,
                notifier,
            }) = receiver.next().await
            {
                if !cache.insert(&block_packet.bytes) {
                    // If the block was already received, we skip it and poll again.
                    trace!("Block already received.");

                    if let Some(notifier) = notifier {
                        if let Err(e) = notifier.send(Err(BlockSubmitterError("block already received".to_string()))) {
                            error!("failed to send error: {:?}.", e);
                        }
                    }

                    metrics.known_blocks_inc();
                    if let Some(peer_id) = from {
                        peer_manager
                            .get_map(&peer_id, |peer| {
                                peer.0.metrics().known_blocks_inc();
                            })
                            .unwrap_or_default();
                    }
                    continue;
                }

                let pow_score = pow.score(&block_packet.bytes);

                if let Err(e) = processor_worker.send(ProcessorWorkerEvent {
                    from,
                    block_packet,
                    pow_score,
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
