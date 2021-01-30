// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    event::LatestSolidMilestoneChanged,
    helper,
    peer::PeerManager,
    storage::StorageBackend,
    worker::{
        IndexUpdaterWorker, IndexUpdaterWorkerEvent, MessageRequesterWorker, MessageRequesterWorkerEvent,
        MetricsWorker, MilestoneRequesterWorker, PeerManagerResWorker, RequestedMessages, RequestedMilestones,
    },
    ProtocolMetrics,
};

use bee_ledger::{LedgerWorker, LedgerWorkerEvent};
use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    MessageId,
};
use bee_runtime::{event::Bus, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{traversal, MsTangle, TangleWorker};

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
    tangle: &MsTangle<B>,
    message_requester: &mpsc::UnboundedSender<MessageRequesterWorkerEvent>,
    requested_messages: &RequestedMessages,
    target_index: MilestoneIndex,
    target_id: MessageId,
) {
    // TODO: This wouldn't be necessary if the traversal code wasn't closure-driven
    let mut missing = Vec::new();

    traversal::visit_parents_depth_first(
        &**tangle,
        target_id,
        |id, _, metadata| async move { !metadata.flags().is_solid() && !requested_messages.contains(&id).await },
        |_, _, _| {},
        |_, _, _| {},
        |missing_id| missing.push(*missing_id),
    )
    .await;

    debug!(
        "Heavy solidification of milestone {} {}: {} messages requested.",
        *target_index,
        target_id,
        missing.len()
    );

    for missing_id in missing {
        helper::request_message(tangle, message_requester, requested_messages, missing_id, target_index).await;
    }
}

async fn solidify<B: StorageBackend>(
    tangle: &MsTangle<B>,
    ledger_worker: &mpsc::UnboundedSender<LedgerWorkerEvent>,
    index_updater_worker: &mpsc::UnboundedSender<IndexUpdaterWorkerEvent>,
    peer_manager: &PeerManager,
    metrics: &ProtocolMetrics,
    bus: &Bus<'static>,
    id: MessageId,
    index: MilestoneIndex,
) {
    debug!("New solid milestone {}.", *index);

    tangle.update_latest_solid_milestone_index(index);

    if let Err(e) = ledger_worker.send(LedgerWorkerEvent(id)) {
        warn!("Sending message_id to ledger worker failed: {}.", e);
    }

    if let Err(e) = index_updater_worker
        // TODO get MS
        .send(IndexUpdaterWorkerEvent(index, Milestone::new(id, 0)))
    {
        warn!("Sending message_id to `IndexUpdater` failed: {:?}.", e);
    }

    helper::broadcast_heartbeat(&peer_manager, &metrics, &tangle).await;

    bus.dispatch(LatestSolidMilestoneChanged {
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
            TypeId::of::<LedgerWorker>(),
            TypeId::of::<IndexUpdaterWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let message_requester = node.worker::<MessageRequesterWorker>().unwrap().tx.clone();
        let milestone_requester = node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone();
        let ledger_worker = node.worker::<LedgerWorker>().unwrap().tx.clone();
        let milestone_cone_updater = node.worker::<IndexUpdaterWorker>().unwrap().tx.clone();
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_messages = node.resource::<RequestedMessages>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let metrics = node.resource::<ProtocolMetrics>();
        let peer_manager = node.resource::<PeerManager>();
        let bus = node.bus();
        let ms_sync_count = config;

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            let mut next = tangle.get_latest_solid_milestone_index() + MilestoneIndex(1);

            while let Some(MilestoneSolidifierWorkerEvent(index)) = receiver.next().await {
                let lsmi = tangle.get_latest_solid_milestone_index();
                let lmi = tangle.get_latest_milestone_index();

                // Request all milestones within a range.
                while next <= cmp::min(lsmi + MilestoneIndex(ms_sync_count), lmi) {
                    helper::request_milestone(&tangle, &milestone_requester, &*requested_milestones, next, None).await;
                    next = next + MilestoneIndex(1);
                }

                if index < next {
                    if let Some(message_id) = tangle.get_milestone_message_id(index).await {
                        if let Some(message) = tangle.get(&message_id).await {
                            debug!("Light solidification of milestone {} {}.", index, message_id);
                            helper::request_message(
                                &tangle,
                                &message_requester,
                                &requested_messages,
                                *message.parent2(),
                                index,
                            )
                            .await;
                        } else {
                            error!("Requested milestone {} message not present in the tangle.", index)
                        }
                    } else {
                        error!("Requested milestone {} message id not present in the tangle.", index)
                    }
                }

                let mut target = lsmi + MilestoneIndex(1);

                while target <= lmi {
                    if let Some(id) = tangle.get_milestone_message_id(target).await {
                        if tangle.is_solid_message(&id).await {
                            solidify(
                                &tangle,
                                &ledger_worker,
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
                            heavy_solidification(&tangle, &message_requester, &requested_messages, target, id).await;
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
