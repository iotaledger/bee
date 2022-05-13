// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, cmp, convert::Infallible};

use async_trait::async_trait;
use bee_block::{
    payload::milestone::{MilestoneId, MilestoneIndex},
    BlockId,
};
use bee_ledger::workers::consensus::{ConsensusWorker, ConsensusWorkerCommand};
use bee_runtime::{event::Bus, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{
    event::SolidMilestoneChanged, milestone_metadata::MilestoneMetadata, traversal, Tangle, TangleWorker,
};
use futures::StreamExt;
use log::{debug, error, info, warn};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    types::metrics::NodeMetrics,
    workers::{
        heartbeater::broadcast_heartbeat,
        peer::PeerManager,
        requester::{request_block, request_milestone},
        storage::StorageBackend,
        BlockRequesterWorker, IndexUpdaterWorker, IndexUpdaterWorkerEvent, MetricsWorker, MilestoneRequesterWorker,
        PeerManagerResWorker, RequestedBlocks, RequestedMilestones,
    },
};

pub(crate) struct MilestoneSolidifierWorkerEvent(pub MilestoneIndex);

pub(crate) struct MilestoneSolidifierWorker {
    pub(crate) tx: mpsc::UnboundedSender<MilestoneSolidifierWorkerEvent>,
}

async fn heavy_solidification<B: StorageBackend>(
    tangle: &Tangle<B>,
    block_requester: &BlockRequesterWorker,
    requested_blocks: &RequestedBlocks,
    target_index: MilestoneIndex,
    target_id: BlockId,
) -> usize {
    // TODO: This wouldn't be necessary if the traversal code wasn't closure-driven
    let mut missing = Vec::new();

    traversal::visit_parents_depth_first(
        tangle,
        target_id,
        |id, _, metadata| !metadata.flags().is_solid() && !requested_blocks.contains(id),
        |_, _, _| {},
        |_, _, _| {},
        |missing_id| missing.push(*missing_id),
    );

    let missing_len = missing.len();

    for missing_id in missing {
        request_block(tangle, block_requester, requested_blocks, missing_id, target_index).await;
    }

    missing_len
}

#[allow(clippy::too_many_arguments)]
fn solidify<B: StorageBackend>(
    tangle: &Tangle<B>,
    consensus_worker: &mpsc::UnboundedSender<ConsensusWorkerCommand>,
    index_updater_worker: &mpsc::UnboundedSender<IndexUpdaterWorkerEvent>,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    bus: &Bus<'static>,
    id: BlockId,
    index: MilestoneIndex,
) {
    debug!("New solid milestone {}.", *index);

    tangle.update_solid_milestone_index(index);

    if let Err(e) = consensus_worker.send(ConsensusWorkerCommand::ConfirmMilestone(id)) {
        warn!("Sending block_id to consensus worker failed: {}.", e);
    }

    if let Err(e) = index_updater_worker
        // TODO get MS
        .send(IndexUpdaterWorkerEvent(
            index,
            MilestoneMetadata::new(id, MilestoneId::null(), 0),
        ))
    {
        warn!("Sending block_id to `IndexUpdater` failed: {:?}.", e);
    }

    broadcast_heartbeat(tangle, peer_manager, metrics);

    bus.dispatch(SolidMilestoneChanged {
        index,
        // TODO get MS
        milestone: MilestoneMetadata::new(id, MilestoneId::null(), 0),
    });
}

#[async_trait]
impl<N: Node> Worker<N> for MilestoneSolidifierWorker
where
    N::Backend: StorageBackend,
{
    type Config = u32;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<BlockRequesterWorker>(),
            TypeId::of::<MilestoneRequesterWorker>(),
            TypeId::of::<TangleWorker>(),
            TypeId::of::<PeerManagerResWorker>(),
            TypeId::of::<MetricsWorker>(),
            TypeId::of::<ConsensusWorker>(),
            TypeId::of::<IndexUpdaterWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let block_requester = node.worker::<BlockRequesterWorker>().unwrap().clone();
        let milestone_requester = node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone();
        let consensus_worker = node.worker::<ConsensusWorker>().unwrap().tx.clone();
        let milestone_cone_updater = node.worker::<IndexUpdaterWorker>().unwrap().tx.clone();
        let tangle = node.resource::<Tangle<N::Backend>>();
        let requested_blocks = node.resource::<RequestedBlocks>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let metrics = node.resource::<NodeMetrics>();
        let peer_manager = node.resource::<PeerManager>();
        let bus = node.bus();
        let milestone_sync_count = config;

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            let mut next = tangle.get_solid_milestone_index() + MilestoneIndex(1);

            while let Some(MilestoneSolidifierWorkerEvent(index)) = receiver.next().await {
                let smi = tangle.get_solid_milestone_index();
                let lmi = tangle.get_latest_milestone_index();

                // Request all milestones within a range.
                while next <= cmp::min(smi + MilestoneIndex(milestone_sync_count), lmi) {
                    request_milestone(&tangle, &milestone_requester, &*requested_milestones, next, None);
                    next = next + MilestoneIndex(1);
                }

                if index < next {
                    if let Some(block_id) = tangle.get_milestone_block_id(index) {
                        if let Some(block) = tangle.get(&block_id) {
                            debug!(
                                "Light solidification of milestone {} {} in [{};{}].",
                                index,
                                block_id,
                                *smi + 1,
                                *next - 1
                            );
                            for parent in block.parents().iter() {
                                request_block(&tangle, &block_requester, &requested_blocks, *parent, index).await;
                            }
                        } else {
                            error!("Requested milestone {} block not present in the tangle.", index)
                        }
                    } else if *index != 0 {
                        error!("Requested milestone {} block id not present in the tangle.", index)
                    }
                }

                let mut target = smi + MilestoneIndex(1);

                while target <= lmi {
                    if let Some(id) = tangle.get_milestone_block_id(target) {
                        if tangle.is_solid_block(&id).await {
                            solidify(
                                &tangle,
                                &consensus_worker,
                                &milestone_cone_updater,
                                &peer_manager,
                                &metrics,
                                &bus,
                                id,
                                target,
                            );
                        } else {
                            // TODO Is this actually necessary ?
                            let missing_len =
                                heavy_solidification(&tangle, &block_requester, &requested_blocks, target, id).await;
                            debug!(
                                "Heavy solidification of milestone {} {}: {} blocks requested in [{};{}].",
                                target,
                                id,
                                missing_len,
                                *smi + 1,
                                *next - 1
                            );
                            break;
                        }
                    } else {
                        break;
                    }
                    target = target + MilestoneIndex(1);
                }
            }

            info!("Stopped.");
        });

        let _ = tx.send(MilestoneSolidifierWorkerEvent(MilestoneIndex(0)));

        Ok(Self { tx })
    }
}
