// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, convert::Infallible, ops::Deref};

use async_trait::async_trait;
use bee_block::{
    payload::{
        milestone::{MilestonePayload, MilestoneValidationError},
        Payload,
    },
    Block, BlockId,
};
use bee_runtime::{event::Bus, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{event::LatestMilestoneChanged, milestone_metadata::MilestoneMetadata, Tangle, TangleWorker};
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    types::{metrics::NodeMetrics, milestone_key_manager::MilestoneKeyManager},
    workers::{
        config::ProtocolConfig, heartbeater::broadcast_heartbeat, peer::PeerManager, storage::StorageBackend,
        MetricsWorker, MilestoneRequesterWorker, MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent,
        PeerManagerResWorker, RequestedMilestones,
    },
};

#[derive(Debug)]
pub(crate) enum Error {
    BlockMilestoneParentsMismatch,
    InvalidMilestone(MilestoneValidationError),
}

pub(crate) struct MilestonePayloadWorkerEvent {
    pub(crate) block_id: BlockId,
    pub(crate) block: Block,
}

pub(crate) struct MilestonePayloadWorker {
    pub(crate) tx: mpsc::UnboundedSender<MilestonePayloadWorkerEvent>,
}

fn validate(
    block_id: BlockId,
    block: &Block,
    milestone: &MilestonePayload,
    key_manager: &MilestoneKeyManager,
) -> Result<MilestoneMetadata, Error> {
    if !block.parents().eq(milestone.essence().parents()) {
        return Err(Error::BlockMilestoneParentsMismatch);
    }

    milestone
        .validate(
            &key_manager
                .get_public_keys(milestone.essence().index())
                .into_iter()
                .collect::<Vec<String>>(),
            key_manager.min_threshold(),
        )
        .map_err(Error::InvalidMilestone)?;

    Ok(MilestoneMetadata::new(
        block_id,
        milestone.id(),
        milestone.essence().timestamp(),
    ))
}

#[allow(clippy::too_many_arguments)]
fn process<B: StorageBackend>(
    tangle: &Tangle<B>,
    block_id: BlockId,
    block: Block,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    requested_milestones: &RequestedMilestones,
    milestone_solidifier: &mpsc::UnboundedSender<MilestoneSolidifierWorkerEvent>,
    key_manager: &MilestoneKeyManager,
    bus: &Bus<'static>,
) {
    if let Some(Payload::Milestone(milestone_payload)) = block.payload() {
        metrics.milestone_payloads_inc(1);

        let index = milestone_payload.essence().index();

        if index <= tangle.get_solid_milestone_index() {
            return;
        }

        match validate(block_id, &block, milestone_payload, key_manager) {
            Ok(milestone_metadata) => {
                tangle.add_milestone(index, milestone_metadata.clone(), milestone_payload.deref().clone());
                if index > tangle.get_latest_milestone_index() {
                    info!("New milestone {} {}.", index, milestone_metadata.block_id());
                    tangle.update_latest_milestone_index(index);

                    broadcast_heartbeat(tangle, peer_manager, metrics);

                    bus.dispatch(LatestMilestoneChanged {
                        index,
                        milestone: milestone_metadata,
                    });
                } else {
                    debug!("New milestone {} {}.", *index, milestone_metadata.block_id());
                }

                requested_milestones.remove(&index);

                if let Err(e) = milestone_solidifier.send(MilestoneSolidifierWorkerEvent(index)) {
                    error!("Sending solidification event failed: {}.", e);
                }
            }
            Err(e) => debug!("Invalid milestone block {}: {:?}.", block_id, e),
        }
    } else {
        error!(
            "Missing or invalid payload for block {}: expected milestone payload.",
            block_id
        );
    }
}

#[async_trait]
impl<N: Node> Worker<N> for MilestonePayloadWorker
where
    N::Backend: StorageBackend,
{
    type Config = ProtocolConfig;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<MilestoneSolidifierWorker>(),
            TypeId::of::<TangleWorker>(),
            TypeId::of::<MilestoneRequesterWorker>(),
            TypeId::of::<PeerManagerResWorker>(),
            TypeId::of::<MetricsWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let milestone_solidifier = node.worker::<MilestoneSolidifierWorker>().unwrap().tx.clone();
        let tangle = node.resource::<Tangle<N::Backend>>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<NodeMetrics>();
        let key_manager = MilestoneKeyManager::new(
            config.coordinator.public_key_count,
            config.coordinator.public_key_ranges.into_boxed_slice(),
        );
        let bus = node.bus();
        let (tx, rx) = mpsc::unbounded_channel();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(MilestonePayloadWorkerEvent { block_id, block }) = receiver.next().await {
                process(
                    &tangle,
                    block_id,
                    block,
                    &peer_manager,
                    &metrics,
                    &requested_milestones,
                    &milestone_solidifier,
                    &key_manager,
                    &bus,
                );
            }

            // Before the worker completely stops, the receiver needs to be drained for milestone payloads to be
            // analysed. Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(MilestonePayloadWorkerEvent { block_id, block })) = receiver.next().now_or_never() {
                process(
                    &tangle,
                    block_id,
                    block,
                    &peer_manager,
                    &metrics,
                    &requested_milestones,
                    &milestone_solidifier,
                    &key_manager,
                    &bus,
                );
                count += 1;
            }

            debug!("Drained {} blocks.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
