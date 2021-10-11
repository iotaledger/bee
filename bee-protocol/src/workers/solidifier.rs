// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::metrics::NodeMetrics,
    workers::{
        heartbeater::broadcast_heartbeat,
        peer::PeerManager,
        requester::{request_message, request_milestone},
        storage::StorageBackend,
        IndexUpdaterWorker, IndexUpdaterWorkerEvent, MessageRequesterWorker, MetricsWorker, MilestoneRequesterWorker,
        PeerManagerResWorker, RequestedMessages, RequestedMilestones,
    },
};

use bee_ledger::workers::consensus::{ConsensusWorker, ConsensusWorkerCommand};
use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    MessageId,
};
use bee_runtime::{event::Bus, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{event::SolidMilestoneChanged, traversal, Tangle, TangleWorker};

use async_trait::async_trait;
use futures::StreamExt;
use log::{debug, error, info, warn};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, cmp, convert::Infallible};

pub(crate) struct MilestoneSolidifierWorkerEvent(pub MilestoneIndex);

pub(crate) struct MilestoneSolidifierWorker {
    pub(crate) tx: mpsc::UnboundedSender<MilestoneSolidifierWorkerEvent>,
}

async fn heavy_solidification<B: StorageBackend>(
    tangle: &Tangle<B>,
    message_requester: &MessageRequesterWorker,
    requested_messages: &RequestedMessages,
    target_index: MilestoneIndex,
    target_id: MessageId,
) -> usize {
    // TODO: This wouldn't be necessary if the traversal code wasn't closure-driven
    let mut missing = Vec::new();

    traversal::visit_parents_depth_first(
        tangle,
        target_id,
        |id, _, metadata| async move { !metadata.flags().is_solid() && !requested_messages.contains(&id).await },
        |_, _, _| {},
        |_, _, _| {},
        |missing_id| missing.push(*missing_id),
    )
    .await;

    let missing_len = missing.len();

    for missing_id in missing {
        request_message(tangle, message_requester, requested_messages, missing_id, target_index).await;
    }

    missing_len
}

#[allow(clippy::too_many_arguments)]
async fn solidify<B: StorageBackend>(
    tangle: &Tangle<B>,
    consensus_worker: &mpsc::UnboundedSender<ConsensusWorkerCommand>,
    index_updater_worker: &mpsc::UnboundedSender<IndexUpdaterWorkerEvent>,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    bus: &Bus<'static>,
    id: MessageId,
    index: MilestoneIndex,
) {
    debug!("New solid milestone {}.", *index);

    tangle.update_solid_milestone_index(index);

    if let Err(e) = consensus_worker.send(ConsensusWorkerCommand::ConfirmMilestone(id)) {
        warn!("Sending message_id to consensus worker failed: {}.", e);
    }

    if let Err(e) = index_updater_worker
        // TODO get MS
        .send(IndexUpdaterWorkerEvent(index, Milestone::new(id, 0)))
    {
        warn!("Sending message_id to `IndexUpdater` failed: {:?}.", e);
    }

    broadcast_heartbeat(&peer_manager, &metrics, &tangle).await;

    bus.dispatch(SolidMilestoneChanged {
        index,
        // TODO get MS
        milestone: Milestone::new(id, 0),
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
            TypeId::of::<MessageRequesterWorker>(),
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
        let message_requester = node.worker::<MessageRequesterWorker>().unwrap().clone();
        let milestone_requester = node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone();
        let consensus_worker = node.worker::<ConsensusWorker>().unwrap().tx.clone();
        let milestone_cone_updater = node.worker::<IndexUpdaterWorker>().unwrap().tx.clone();
        let tangle = node.resource::<Tangle<N::Backend>>();
        let requested_messages = node.resource::<RequestedMessages>();
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
                    request_milestone(&tangle, &milestone_requester, &*requested_milestones, next, None).await;
                    next = next + MilestoneIndex(1);
                }

                if index < next {
                    if let Some(message_id) = tangle.get_milestone_message_id(index).await {
                        if let Some(message) = tangle.get(&message_id).await {
                            debug!(
                                "Light solidification of milestone {} {} in [{};{}].",
                                index,
                                message_id,
                                *smi + 1,
                                *next - 1
                            );
                            for parent in message.parents().iter() {
                                request_message(&tangle, &message_requester, &requested_messages, *parent, index).await;
                            }
                        } else {
                            error!("Requested milestone {} message not present in the tangle.", index)
                        }
                    } else if *index != 0 {
                        error!("Requested milestone {} message id not present in the tangle.", index)
                    }
                }

                let mut target = smi + MilestoneIndex(1);

                while target <= lmi {
                    if let Some(id) = tangle.get_milestone_message_id(target).await {
                        if tangle.is_solid_message(&id).await {
                            solidify(
                                &tangle,
                                &consensus_worker,
                                &milestone_cone_updater,
                                &peer_manager,
                                &metrics,
                                &bus,
                                id,
                                target,
                            )
                            .await;
                        } else {
                            // TODO Is this actually necessary ?
                            let missing_len =
                                heavy_solidification(&tangle, &message_requester, &requested_messages, target, id)
                                    .await;
                            debug!(
                                "Heavy solidification of milestone {} {}: {} messages requested in [{};{}].",
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
