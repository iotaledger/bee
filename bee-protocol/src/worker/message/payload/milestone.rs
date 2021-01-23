// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::ProtocolConfig,
    event::LatestMilestoneChanged,
    helper,
    key_manager::KeyManager,
    peer::PeerManager,
    storage::StorageBackend,
    worker::{
        MetricsWorker, MilestoneRequesterWorker, MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent,
        PeerManagerResWorker, RequestedMilestones, TangleWorker,
    },
    ProtocolMetrics,
};

use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    payload::{milestone::MilestoneValidationError, Payload},
    MessageId,
};
use bee_runtime::{event::Bus, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::MsTangle;

use async_trait::async_trait;
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, convert::Infallible};

#[derive(Debug)]
pub(crate) enum Error {
    UnknownMessage,
    NoMilestonePayload,
    Parent1Mismatch(MessageId, MessageId),
    Parent2Mismatch(MessageId, MessageId),
    InvalidMilestone(MilestoneValidationError),
}

#[derive(Debug)]
pub(crate) struct MilestonePayloadWorkerEvent(pub(crate) MessageId);

pub(crate) struct MilestonePayloadWorker {
    pub(crate) tx: mpsc::UnboundedSender<MilestonePayloadWorkerEvent>,
}

async fn validate<B: StorageBackend>(
    tangle: &MsTangle<B>,
    key_manager: &KeyManager,
    message_id: MessageId,
) -> Result<(MilestoneIndex, Milestone), Error> {
    let message = tangle.get(&message_id).await.ok_or(Error::UnknownMessage)?;

    match message.payload() {
        Some(Payload::Milestone(milestone)) => {
            if message.parent1() != milestone.essence().parent1() {
                return Err(Error::Parent1Mismatch(
                    *message.parent1(),
                    *milestone.essence().parent1(),
                ));
            }
            if message.parent2() != milestone.essence().parent2() {
                return Err(Error::Parent2Mismatch(
                    *message.parent2(),
                    *milestone.essence().parent2(),
                ));
            }

            milestone
                .validate(
                    &key_manager
                        .get_public_keys(milestone.essence().index().into())
                        .into_iter()
                        .collect::<Vec<String>>(),
                    key_manager.min_threshold(),
                )
                .map_err(|e| Error::InvalidMilestone(e))?;

            Ok((
                MilestoneIndex(milestone.essence().index()),
                Milestone::new(message_id, milestone.essence().timestamp()),
            ))
        }
        _ => Err(Error::NoMilestonePayload),
    }
}

async fn process<B: StorageBackend>(
    tangle: &MsTangle<B>,
    message_id: MessageId,
    peer_manager: &PeerManager,
    metrics: &ProtocolMetrics,
    requested_milestones: &RequestedMilestones,
    milestone_solidifier: &mpsc::UnboundedSender<MilestoneSolidifierWorkerEvent>,
    key_manager: &KeyManager,
    bus: &Bus<'static>,
) {
    if let Some(meta) = tangle.get_metadata(&message_id).await {
        if meta.flags().is_milestone() {
            return;
        }
    }
    match validate(&tangle, &key_manager, message_id).await {
        Ok((index, milestone)) => {
            tangle.add_milestone(index, milestone.clone()).await;
            if index > tangle.get_latest_milestone_index() {
                info!("New milestone {} {}.", *index, milestone.message_id());
                tangle.update_latest_milestone_index(index);

                helper::broadcast_heartbeat(
                    &peer_manager,
                    &metrics,
                    tangle.get_latest_solid_milestone_index(),
                    tangle.get_pruning_index(),
                    index,
                )
                .await;

                bus.dispatch(LatestMilestoneChanged {
                    index,
                    milestone: milestone.clone(),
                });
            }

            requested_milestones.remove(&index).await;

            if let Err(e) = milestone_solidifier.send(MilestoneSolidifierWorkerEvent(index)) {
                error!("Sending solidification event failed: {}.", e);
            }
        }
        Err(e) => debug!("Invalid milestone message: {:?}.", e),
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
        let (tx, rx) = mpsc::unbounded_channel();
        let milestone_solidifier = node.worker::<MilestoneSolidifierWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<ProtocolMetrics>();
        let key_manager = KeyManager::new(
            config.coordinator.public_key_count,
            config.coordinator.public_key_ranges.into_boxed_slice(),
        );
        let bus = node.bus();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(MilestonePayloadWorkerEvent(message_id)) = receiver.next().await {
                process(
                    &tangle,
                    message_id,
                    &peer_manager,
                    &metrics,
                    &requested_milestones,
                    &milestone_solidifier,
                    &key_manager,
                    &bus,
                )
                .await;
            }

            // Before the worker completely stops, the receiver needs to be drained for milestone payloads to be
            // analysed. Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(MilestonePayloadWorkerEvent(message_id))) = receiver.next().now_or_never() {
                process(
                    &tangle,
                    message_id,
                    &peer_manager,
                    &metrics,
                    &requested_milestones,
                    &milestone_solidifier,
                    &key_manager,
                    &bus,
                )
                .await;
                count += 1;
            }

            debug!("Drained {} messages.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
