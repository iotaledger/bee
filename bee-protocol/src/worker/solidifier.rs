// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    event::LatestSolidMilestoneChanged,
    helper,
    peer::PeerManager,
    storage::StorageBackend,
    worker::{
        MessageRequesterWorker, MessageRequesterWorkerEvent, MetricsWorker, MilestoneConeUpdaterWorker,
        MilestoneConeUpdaterWorkerEvent, MilestoneRequesterWorker, PeerManagerResWorker, RequestedMessages,
        RequestedMilestones, TangleWorker,
    },
    ProtocolMetrics,
};

use bee_ledger::{LedgerWorker, LedgerWorkerEvent};
use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    MessageId,
};
use bee_network::NetworkController;
use bee_runtime::{event::Bus, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{traversal, MsTangle};

use async_trait::async_trait;
use futures::StreamExt;
use log::{debug, info, warn};
use tokio::sync::mpsc;

use std::{any::TypeId, convert::Infallible};

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

async fn light_solidification<B: StorageBackend>(
    tangle: &MsTangle<B>,
    message_requester: &mpsc::UnboundedSender<MessageRequesterWorkerEvent>,
    requested_messages: &RequestedMessages,
    target_index: MilestoneIndex,
    target_id: MessageId,
) {
    debug!("Light solidification of milestone {}.", *target_index);

    if let Some((parent1, parent2)) = tangle.get(&target_id).await.map(|m| (*m.parent1(), *m.parent2())) {
        helper::request_message(tangle, message_requester, requested_messages, parent1, target_index).await;
        helper::request_message(tangle, message_requester, requested_messages, parent2, target_index).await;
    }
}

fn solidify<B: StorageBackend>(
    tangle: &MsTangle<B>,
    ledger_worker: &mpsc::UnboundedSender<LedgerWorkerEvent>,
    milestone_cone_updater: &mpsc::UnboundedSender<MilestoneConeUpdaterWorkerEvent>,
    peer_manager: &PeerManager,
    metrics: &ProtocolMetrics,
    network: &NetworkController,
    bus: &Bus<'static>,
    id: MessageId,
    index: MilestoneIndex,
) {
    debug!("New solid milestone {}.", *index);

    tangle.update_latest_solid_milestone_index(index);

    if let Err(e) = ledger_worker.send(LedgerWorkerEvent(id)) {
        warn!("Sending message_id to ledger worker failed: {}.", e);
    }

    if let Err(e) = milestone_cone_updater
        // TODO get MS
        .send(MilestoneConeUpdaterWorkerEvent(index, Milestone::new(id, 0)))
    {
        warn!("Sending message_id to `MilestoneConeUpdater` failed: {:?}.", e);
    }

    helper::broadcast_heartbeat(
        &peer_manager,
        &network,
        &metrics,
        index,
        tangle.get_pruning_index(),
        tangle.get_latest_milestone_index(),
    );

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
            TypeId::of::<MilestoneConeUpdaterWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let message_requester = node.worker::<MessageRequesterWorker>().unwrap().tx.clone();
        let milestone_requester = node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone();
        let ledger_worker = node.worker::<LedgerWorker>().unwrap().tx.clone();
        let milestone_cone_updater = node.worker::<MilestoneConeUpdaterWorker>().unwrap().tx.clone();
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_messages = node.resource::<RequestedMessages>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let metrics = node.resource::<ProtocolMetrics>();
        let peer_manager = node.resource::<PeerManager>();
        let network = node.resource::<NetworkController>();
        let bus = node.bus();
        let ms_sync_count = config;

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            let mut requested = tangle.get_latest_solid_milestone_index() + MilestoneIndex(1);

            while let Some(MilestoneSolidifierWorkerEvent(index)) = receiver.next().await {
                let lsmi = tangle.get_latest_solid_milestone_index();
                let lmi = tangle.get_latest_milestone_index();

                while requested <= lmi && *(requested - lsmi) <= ms_sync_count {
                    if let Some(id) = tangle.get_milestone_message_id(requested).await {
                        let heavy = tangle
                            .get_metadata(&id)
                            .await
                            .map(|m| !m.flags().is_requested())
                            .unwrap_or(false);

                        if heavy {
                            heavy_solidification(&tangle, &message_requester, &requested_messages, requested, id).await;
                        } else {
                            light_solidification(&tangle, &message_requester, &requested_messages, requested, id).await;
                        }
                    } else {
                        helper::request_milestone(
                            &tangle,
                            &milestone_requester,
                            &*requested_milestones,
                            requested,
                            None,
                        )
                        .await;
                    }
                    requested = requested + MilestoneIndex(1);
                }

                // TODO handle else
                if let Some(id) = tangle.get_milestone_message_id(index).await {
                    if tangle.is_solid_message(&id).await {
                        solidify(
                            &tangle,
                            &ledger_worker,
                            &milestone_cone_updater,
                            &peer_manager,
                            &metrics,
                            &network,
                            &bus,
                            id,
                            index,
                        );
                    } else if tangle.is_synced_threshold(2) {
                        heavy_solidification(&tangle, &message_requester, &requested_messages, index, id).await;
                    } else if index > lsmi && index <= lsmi + MilestoneIndex(ms_sync_count) {
                        light_solidification(&tangle, &message_requester, &requested_messages, index, id).await;
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
