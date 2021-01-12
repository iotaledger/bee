// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    event::LatestSolidMilestoneChanged,
    helper,
    peer::PeerManager,
    storage::StorageBackend,
    worker::{
        MessageRequesterWorker, MessageRequesterWorkerEvent, MetricsWorker, MilestoneRequesterWorker,
        PeerManagerResWorker, RequestedMessages, RequestedMilestones, TangleWorker,
    },
    ProtocolMetrics,
};

use bee_ledger::{LedgerWorker, LedgerWorkerEvent};
use bee_message::MessageId;
use bee_network::NetworkController;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{milestone::MilestoneIndex, traversal, MsTangle};

use async_trait::async_trait;
use futures::{stream::FusedStream, StreamExt};
use log::{debug, info, warn};
use tokio::{sync::mpsc, task::spawn, time::interval};

use std::{any::TypeId, convert::Infallible, time::Duration};

const KICKSTART_INTERVAL_SEC: u64 = 1;

pub(crate) struct MilestoneSolidifierWorkerEvent(pub MilestoneIndex);

pub(crate) struct MilestoneSolidifierWorker {
    pub(crate) tx: mpsc::UnboundedSender<MilestoneSolidifierWorkerEvent>,
}

async fn trigger_solidification<B: StorageBackend>(
    tangle: &MsTangle<B>,
    message_requester: &mpsc::UnboundedSender<MessageRequesterWorkerEvent>,
    requested_messages: &RequestedMessages,
    target_index: MilestoneIndex,
    target_id: MessageId,
) {
    debug!("Triggering solidification for milestone {}.", *target_index);

    // TODO: This wouldn't be necessary if the traversal code wasn't closure-driven
    let mut missing = Vec::new();

    traversal::visit_parents_depth_first(
        &**tangle,
        target_id,
        |id, _, metadata| {
            (!metadata.flags().is_requested() || *id == target_id)
                && !metadata.flags().is_solid()
                && !requested_messages.contains_key(&id)
        },
        |_, _, _| {},
        |_, _, _| {},
        |missing_id| missing.push(*missing_id),
    )
    .await;

    for missing_id in missing {
        helper::request_message(tangle, message_requester, requested_messages, missing_id, target_index).await;
    }
}

fn save_index(target_index: MilestoneIndex, queue: &mut Vec<MilestoneIndex>) {
    debug!("Storing milestone {}.", *target_index);
    if let Err(pos) = queue.binary_search_by(|index| target_index.cmp(index)) {
        queue.insert(pos, target_index);
    }
}

// bus.dispatch(LatestSolidMilestoneChanged {
//     index,
//     milestone: milestone.clone(),
// });
// if let Err(e) = milestone_cone_updater
//     .send(MilestoneConeUpdaterWorkerEvent(index, milestone.clone()))
// {
//     error!("Sending message id to milestone validation failed: {:?}.", e);
// }

// // TODO we need to get the milestone from the tangle to dispatch it.
// // At the time of writing, the tangle only contains an index <-> id mapping.
// // timestamp is obviously wrong in thr meantime.
// bus.dispatch(LatestSolidMilestoneChanged {
//     index,
//     milestone: Milestone::new(*message_id, 0),
// });
// // TODO we need to get the milestone from the tangle to dispatch it.
// // At the time of writing, the tangle only contains an index <-> id mapping.
// // timestamp is obviously wrong in thr meantime.
// if let Err(e) = milestone_cone_updater
//     .send(MilestoneConeUpdaterWorkerEvent(index, Milestone::new(*message_id, 0)))
// {
//     error!("Sending message_id to `MilestoneConeUpdater` failed: {:?}.", e);
// }

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
            // TypeId::of::<MilestoneRequesterWorker>(),
            TypeId::of::<PeerManagerResWorker>(),
            TypeId::of::<MetricsWorker>(),
            TypeId::of::<LedgerWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let message_requester = node.worker::<MessageRequesterWorker>().unwrap().tx.clone();
        let milestone_requester = node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone();
        let ledger_worker = node.worker::<LedgerWorker>().unwrap().tx.clone();
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_messages = node.resource::<RequestedMessages>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let metrics = node.resource::<ProtocolMetrics>();
        let peer_manager = node.resource::<PeerManager>();
        let network = node.resource::<NetworkController>();
        let ms_sync_count = config;
        // let milestone_solidifier = tx.clone();
        // let mut next_ms = tangle.get_latest_solid_milestone_index() + MilestoneIndex(1);
        //
        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            let mut requested = tangle.get_latest_solid_milestone_index() + MilestoneIndex(1);

            while let Some(MilestoneSolidifierWorkerEvent(index)) = receiver.next().await {
                let lsmi = tangle.get_latest_solid_milestone_index();
                let lmi = tangle.get_latest_milestone_index();

                while requested < lmi && *(requested - lsmi) <= ms_sync_count {
                    helper::request_milestone(&tangle, &milestone_requester, &*requested_milestones, requested, None)
                        .await;
                    requested = requested + MilestoneIndex(1);
                }

                //TODO handle else
                if let Some(id) = tangle.get_milestone_message_id(index).await {
                    if tangle.is_solid_message(&id).await {
                        debug!("New solid milestone {}.", *index);

                        tangle.update_latest_solid_milestone_index(index);

                        if let Err(e) = ledger_worker.send(LedgerWorkerEvent(id)) {
                            warn!("Sending message_id to ledger worker failed: {}.", e);
                        }

                        helper::broadcast_heartbeat(
                            &peer_manager,
                            &network,
                            &metrics,
                            index,
                            tangle.get_pruning_index(),
                            tangle.get_latest_milestone_index(),
                        );
                    } else if tangle.is_synced_threshold(2)
                        || index > lsmi && index <= lsmi + MilestoneIndex(ms_sync_count)
                    {
                        trigger_solidification(&tangle, &message_requester, &requested_messages, index, id).await;
                    }
                }
            }

            info!("Stopped.");
        });

        //     while receiver.next().await.is_some() {
        //         let latest_ms = tangle.get_latest_milestone_index();
        //         next_ms = tangle.get_latest_solid_milestone_index() + MilestoneIndex(1);
        //
        //         if !peer_manager.is_empty() && *next_ms + ms_sync_count < *latest_ms {
        //             for index in *next_ms..(*next_ms + ms_sync_count) {
        //
        //             }
        //
        //             break;
        //         }
        //     }
        //
        //     while let Some(MilestoneSolidifierWorkerEvent(index)) = receiver.next().await {
        //         save_index(index, &mut queue);
        //         while let Some(index) = queue.pop() {
        //             if index == next_ms {
        //                 trigger_solidification_unchecked(
        //                     &tangle,
        //                     &message_requester,
        //                     &*requested_messages,
        //                     index,
        //                     &mut next_ms,
        //                 )
        //                 .await;
        //             } else {
        //                 queue.push(index);
        //                 break;
        //             }
        //         }
        //     }
        //

        Ok(Self { tx })
    }
}
