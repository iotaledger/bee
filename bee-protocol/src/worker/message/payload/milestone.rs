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
use bee_network::NetworkController;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::MsTangle;

use async_trait::async_trait;
use futures::stream::StreamExt;
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

async fn validate<N: Node>(
    tangle: &MsTangle<N::Backend>,
    key_manager: &KeyManager,
    message_id: MessageId,
) -> Result<(MilestoneIndex, Milestone), Error>
where
    N::Backend: StorageBackend,
{
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
        let network = node.resource::<NetworkController>();
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
                if let Some(meta) = tangle.get_metadata(&message_id).await {
                    if meta.flags().is_milestone() {
                        continue;
                    }
                }
                match validate::<N>(&tangle, &key_manager, message_id).await {
                    Ok((index, milestone)) => {
                        tangle.add_milestone(index, milestone.clone()).await;
                        if index > tangle.get_latest_milestone_index() {
                            info!("New milestone {} {}.", *index, milestone.message_id());
                            tangle.update_latest_milestone_index(index);

                            helper::broadcast_heartbeat(
                                &peer_manager,
                                &network,
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

                        if requested_milestones.remove(&index).await.is_some() {
                            tangle
                                .update_metadata(milestone.message_id(), |meta| meta.flags_mut().set_requested(true))
                                .await;
                        }

                        if let Err(e) = milestone_solidifier.send(MilestoneSolidifierWorkerEvent(index)) {
                            error!("Sending solidification event failed: {}.", e);
                        }
                    }
                    Err(e) => debug!("Invalid milestone message: {:?}.", e),
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
