// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    event::LatestSolidMilestoneChanged,
    helper,
    milestone::MilestoneIndex,
    peer::PeerManager,
    tangle::MsTangle,
    worker::{
        MessageRequesterWorker, MessageRequesterWorkerEvent, MetricsWorker, MilestoneRequesterWorker,
        PeerManagerResWorker, RequestedMessages, RequestedMilestones, TangleWorker,
    },
    ProtocolMetrics,
};

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_pt2::{node::Node, worker::Worker};
use bee_network::NetworkController;
use bee_storage::storage::Backend;
use bee_tangle::traversal;

use async_trait::async_trait;
use futures::{stream::FusedStream, StreamExt};
use log::{debug, info, warn};
use tokio::{sync::mpsc, time::interval};

use std::{any::TypeId, convert::Infallible, time::Duration};

const KICKSTART_INTERVAL_SEC: u64 = 1;

pub(crate) struct MilestoneSolidifierWorkerEvent(pub MilestoneIndex);

pub(crate) struct MilestoneSolidifierWorker {
    pub(crate) tx: mpsc::UnboundedSender<MilestoneSolidifierWorkerEvent>,
}

async fn trigger_solidification_unchecked<B: Backend>(
    tangle: &MsTangle<B>,
    message_requester: &mpsc::UnboundedSender<MessageRequesterWorkerEvent>,
    requested_messages: &RequestedMessages,
    target_index: MilestoneIndex,
    next_index: &mut MilestoneIndex,
) {
    if let Some(target_id) = tangle.get_milestone_message_id(target_index) {
        if !tangle.is_solid_message(&target_id) {
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
            );

            for missing_id in missing {
                helper::request_message(tangle, message_requester, requested_messages, missing_id, target_index).await;
            }
        }
        *next_index = target_index + MilestoneIndex(1);
    }
}

fn save_index(target_index: MilestoneIndex, queue: &mut Vec<MilestoneIndex>) {
    debug!("Storing milestone {}.", *target_index);
    if let Err(pos) = queue.binary_search_by(|index| target_index.cmp(index)) {
        queue.insert(pos, target_index);
    }
}

#[async_trait]
impl<N: Node> Worker<N> for MilestoneSolidifierWorker {
    type Config = u32;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<MessageRequesterWorker>(),
            TypeId::of::<MilestoneRequesterWorker>(),
            TypeId::of::<TangleWorker>(),
            TypeId::of::<MilestoneRequesterWorker>(),
            TypeId::of::<PeerManagerResWorker>(),
            TypeId::of::<MetricsWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let message_requester = node.worker::<MessageRequesterWorker>().unwrap().tx.clone();
        let milestone_requester = node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone();
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_messages = node.resource::<RequestedMessages>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let peer_manager = node.resource::<PeerManager>();
        let ms_sync_count = config;
        let milestone_solidifier = tx.clone();
        let mut next_ms = tangle.get_latest_solid_milestone_index() + MilestoneIndex(1);

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, interval(Duration::from_secs(KICKSTART_INTERVAL_SEC)));

            while receiver.next().await.is_some() {
                let latest_ms = tangle.get_latest_milestone_index();
                next_ms = tangle.get_latest_solid_milestone_index() + MilestoneIndex(1);

                if !peer_manager.is_empty() && *next_ms + ms_sync_count < *latest_ms {
                    for index in *next_ms..(*next_ms + ms_sync_count) {
                        helper::request_milestone(
                            &tangle,
                            &milestone_requester,
                            &*requested_milestones,
                            MilestoneIndex(index),
                            None,
                        );
                    }

                    break;
                }
            }

            if receiver.is_terminated() {
                return;
            }

            let (shutdown, _) = receiver.split();
            let mut receiver = ShutdownStream::from_fused(shutdown, rx.fuse());
            let mut queue = vec![];

            while let Some(MilestoneSolidifierWorkerEvent(index)) = receiver.next().await {
                save_index(index, &mut queue);
                while let Some(index) = queue.pop() {
                    if index == next_ms {
                        trigger_solidification_unchecked(
                            &tangle,
                            &message_requester,
                            &*requested_messages,
                            index,
                            &mut next_ms,
                        )
                        .await;
                    } else {
                        queue.push(index);
                        break;
                    }
                }
            }

            info!("Stopped.");
        });

        let milestone_requester = node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone();
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let network = node.resource::<NetworkController>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<ProtocolMetrics>();

        node.bus()
            .add_listener::<Self, _, _>(move |latest_solid_milestone: &LatestSolidMilestoneChanged| {
                debug!("New solid milestone {}.", *latest_solid_milestone.index);
                tangle.update_latest_solid_milestone_index(latest_solid_milestone.index);

                let next_ms = latest_solid_milestone.index + MilestoneIndex(ms_sync_count);

                if tangle.contains_milestone(next_ms) {
                    if let Err(e) = milestone_solidifier.send(MilestoneSolidifierWorkerEvent(next_ms)) {
                        warn!("Sending solidification event failed: {}", e);
                    }
                } else {
                    helper::request_milestone(&tangle, &milestone_requester, &*requested_milestones, next_ms, None);
                }

                helper::broadcast_heartbeat(
                    &peer_manager,
                    &network,
                    &metrics,
                    latest_solid_milestone.index,
                    tangle.get_pruning_index(),
                    tangle.get_latest_milestone_index(),
                );
            });

        Ok(Self { tx })
    }
}
