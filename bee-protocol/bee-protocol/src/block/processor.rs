// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, convert::Infallible, time::Instant};

use async_trait::async_trait;
use bee_block::{payload::Payload, Block, BlockId};
use bee_gossip::PeerId;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{block_metadata::BlockMetadata, Tangle, TangleWorker};
use futures::{channel::oneshot::Sender, stream::StreamExt};
use log::{error, info, trace};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    block::submitter::{notify_block, notify_invalid_block},
    event::{BlockProcessed, VertexCreated},
    packets::BlockPacket,
    peer::PeerManager,
    requester::request_block,
    storage::StorageBackend,
    types::metrics::NodeMetrics,
    BlockRequesterWorker, BlockSubmitterError, BroadcasterWorker, BroadcasterWorkerEvent, MetricsWorker, PayloadWorker,
    PayloadWorkerEvent, PeerManagerResWorker, PropagatorWorker, PropagatorWorkerEvent, RequestedBlocks,
    UnreferencedBlockInserterWorker, UnreferencedBlockInserterWorkerEvent,
};

pub(crate) struct ProcessorWorkerEvent {
    pub(crate) from: Option<PeerId>,
    pub(crate) block_packet: BlockPacket,
    pub(crate) pow_score: f64,
    pub(crate) notifier: Option<Sender<Result<BlockId, BlockSubmitterError>>>,
}

pub(crate) struct ProcessorWorker {
    pub(crate) tx: mpsc::UnboundedSender<ProcessorWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for ProcessorWorker
where
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<PropagatorWorker>(),
            TypeId::of::<BroadcasterWorker>(),
            TypeId::of::<BlockRequesterWorker>(),
            TypeId::of::<MetricsWorker>(),
            TypeId::of::<PeerManagerResWorker>(),
            TypeId::of::<PayloadWorker>(),
            TypeId::of::<UnreferencedBlockInserterWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, _: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let propagator = node.worker::<PropagatorWorker>().unwrap().tx.clone();
        let broadcaster = node.worker::<BroadcasterWorker>().unwrap().tx.clone();
        let block_requester = node.worker::<BlockRequesterWorker>().unwrap().clone();
        let payload_worker = node.worker::<PayloadWorker>().unwrap().tx.clone();
        let unreferenced_inserted_worker = node.worker::<UnreferencedBlockInserterWorker>().unwrap().tx.clone();

        let tangle = node.resource::<Tangle<N::Backend>>();
        let requested_blocks = node.resource::<RequestedBlocks>();
        let metrics = node.resource::<NodeMetrics>();
        let peer_manager = node.resource::<PeerManager>();
        let bus = node.bus();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut latency_num: u64 = 0;
            let mut latency_sum: u64 = 0;
            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            let (tx, rx) = async_channel::unbounded();

            // TODO: this is obviously wrong but can't be done properly until the snapshot PR is merged.
            // The node can't work properly with this.
            // @thibault-martinez.
            let protocol_parameters = bee_block::protocol::ProtocolParameters::default();

            for _ in 0..16 {
                let rx = rx.clone();
                let propagator = propagator.clone();
                let broadcaster = broadcaster.clone();
                let block_requester = block_requester.clone();
                let payload_worker = payload_worker.clone();
                let unreferenced_inserted_worker = unreferenced_inserted_worker.clone();
                let tangle = tangle.clone();
                let requested_blocks = requested_blocks.clone();
                let metrics = metrics.clone();
                let peer_manager = peer_manager.clone();
                let bus = bus.clone();
                let protocol_parameters = protocol_parameters.clone();

                tokio::spawn(async move {
                    'next_event: while let Ok(ProcessorWorkerEvent {
                        from,
                        block_packet,
                        pow_score,
                        notifier,
                    }) = rx.recv().await
                    {
                        trace!("Processing received block...");

                        let block = match Block::unpack_strict(&block_packet.bytes[..], &protocol_parameters) {
                            Ok(block) => block,
                            Err(e) => {
                                notify_invalid_block(format!("Invalid block: {:?}.", e), &metrics, notifier);
                                continue;
                            }
                        };

                        if let Some(Payload::Milestone(_)) = block.payload() {
                            if block.nonce() != 0 {
                                notify_invalid_block(
                                    format!("Non-zero milestone nonce: {}.", block.nonce()),
                                    &metrics,
                                    notifier,
                                );
                                continue;
                            }
                        } else if pow_score < protocol_parameters.min_pow_score() as f64 {
                            notify_invalid_block(
                                format!(
                                    "Insufficient pow score: {} < {}.",
                                    pow_score,
                                    protocol_parameters.min_pow_score()
                                ),
                                &metrics,
                                notifier,
                            );
                            continue;
                        }

                        let block_id = block.id();

                        if tangle.contains(&block_id) {
                            metrics.known_blocks_inc();
                            if let Some(ref peer_id) = from {
                                peer_manager
                                    .get_map(peer_id, |peer| {
                                        peer.0.metrics().known_blocks_inc();
                                    })
                                    .unwrap_or_default();
                            }
                            continue 'next_event;
                        } else {
                            let metadata = BlockMetadata::arrived();
                            // There is no data race here even if the `Block` and
                            // `BlockMetadata` are inserted between the call to `tangle.contains`
                            // and here because:
                            // - Both `Block`s are the same because they have the same hash.
                            // - `BlockMetadata` is not overwritten.
                            // - Some extra code is executing due to not calling `continue` but
                            // this does not create inconsistencies.
                            tangle.insert(&block, &block_id, &metadata);
                        }

                        // Send the propagation event ASAP to allow the propagator to do its thing
                        if let Err(e) = propagator.send(PropagatorWorkerEvent(block_id)) {
                            error!("Failed to send block id {} to propagator: {:?}.", block_id, e);
                        }

                        match requested_blocks.remove(&block_id) {
                            // Block was requested.
                            Some((index, instant)) => {
                                latency_num += 1;
                                latency_sum += (Instant::now() - instant).as_millis() as u64;
                                metrics.blocks_average_latency_set(latency_sum / latency_num);

                                for parent in block.parents().iter() {
                                    request_block(&tangle, &block_requester, &requested_blocks, *parent, index).await;
                                }
                            }
                            // Block was not requested.
                            None => {
                                if let Err(e) = broadcaster.send(BroadcasterWorkerEvent {
                                    source: from,
                                    block: block_packet,
                                }) {
                                    error!("Broadcasting block failed: {}.", e);
                                }
                                if let Err(e) = unreferenced_inserted_worker.send(UnreferencedBlockInserterWorkerEvent(
                                    block_id,
                                    tangle.get_latest_milestone_index(),
                                )) {
                                    error!("Sending block to unreferenced inserter failed: {}.", e);
                                }
                            }
                        };

                        let parents = block.parents().to_vec();

                        if payload_worker.send(PayloadWorkerEvent { block_id, block }).is_err() {
                            error!("Sending block {} to payload worker failed.", block_id);
                        }

                        notify_block(block_id, notifier);

                        bus.dispatch(BlockProcessed { block_id });

                        // TODO: boolean values are false at this point in time? trigger event from another location?
                        bus.dispatch(VertexCreated {
                            block_id,
                            parents,
                            is_solid: false,
                            is_referenced: false,
                            is_conflicting: false,
                            is_milestone: false,
                            is_tip: false,
                            is_selected: false,
                        });

                        metrics.new_blocks_inc();
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
