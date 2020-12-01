// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::ProtocolConfig,
    event::{LatestMilestoneChanged, LatestSolidMilestoneChanged},
    milestone::{key_manager::KeyManager, Milestone, MilestoneIndex},
    tangle::MsTangle,
    worker::{
        MilestoneConeUpdaterWorker, MilestoneConeUpdaterWorkerEvent, MilestoneRequesterWorker,
        MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent, RequestedMilestones, TangleWorker,
    },
};

use bee_common::{event::Bus, node::Node, packable::Packable, shutdown_stream::ShutdownStream, worker::Worker};
use bee_message::{payload::Payload, MessageId};

use async_trait::async_trait;
use crypto::ed25519::{verify, PublicKey, Signature};
use futures::stream::StreamExt;
use log::{debug, error, info};

use std::{
    any::TypeId,
    convert::{Infallible, TryInto},
};

#[derive(Debug)]
pub(crate) enum Error {
    UnknownMessage,
    NoMilestonePayload,
    Parent1Mismatch(MessageId, MessageId),
    Parent2Mismatch(MessageId, MessageId),
    InvalidMinThreshold(usize),
    TooFewSignatures(usize, usize),
    SignaturesPublicKeysCountMismatch(usize, usize),
    InsufficientApplicablePublicKeys(usize, usize),
    UnapplicablePublicKey(String),
    InvalidSignature(usize, String),
}

pub(crate) struct MilestoneValidatorWorkerEvent(pub(crate) MessageId);

pub(crate) struct MilestoneValidatorWorker {
    pub(crate) tx: flume::Sender<MilestoneValidatorWorkerEvent>,
}

async fn validate<N: Node>(
    tangle: &MsTangle<N::Backend>,
    key_manager: &KeyManager,
    message_id: MessageId,
) -> Result<Milestone, Error>
where
    N: Node,
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
            if key_manager.min_threshold() == 0 {
                return Err(Error::InvalidMinThreshold(0));
            }
            if milestone.signatures().is_empty() || milestone.signatures().len() < key_manager.min_threshold() {
                return Err(Error::TooFewSignatures(
                    key_manager.min_threshold(),
                    milestone.signatures().len(),
                ));
            }
            if milestone.signatures().len() != milestone.essence().public_keys().len() {
                return Err(Error::SignaturesPublicKeysCountMismatch(
                    milestone.signatures().len(),
                    milestone.essence().public_keys().len(),
                ));
            }

            let public_keys = key_manager.get_public_keys(milestone.essence().index().into());

            if public_keys.len() < key_manager.min_threshold() {
                return Err(Error::InsufficientApplicablePublicKeys(
                    key_manager.min_threshold(),
                    public_keys.len(),
                ));
            }

            let essence_bytes = milestone.essence().pack_new();

            for (index, public_key) in milestone.essence().public_keys().iter().enumerate() {
                // TODO use concrete ED25 types ?
                if !public_keys.contains(&hex::encode(public_key)) {
                    return Err(Error::UnapplicablePublicKey(hex::encode(public_key)));
                }

                // TODO unwrap
                let ed25519_public_key = PublicKey::from_compressed_bytes(*public_key).unwrap();
                // TODO unwrap
                let ed25519_signature =
                    Signature::from_bytes(milestone.signatures()[index].as_ref().try_into().unwrap());

                if !verify(&ed25519_public_key, &ed25519_signature, &essence_bytes) {
                    return Err(Error::InvalidSignature(index, hex::encode(public_key)));
                }
            }

            Ok(Milestone {
                index: MilestoneIndex(milestone.essence().index()),
                message_id,
            })
        }
        _ => Err(Error::NoMilestonePayload),
    }
}

#[async_trait]
impl<N> Worker<N> for MilestoneValidatorWorker
where
    N: Node,
{
    type Config = ProtocolConfig;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<MilestoneSolidifierWorker>(),
            TypeId::of::<MilestoneConeUpdaterWorker>(),
            TypeId::of::<TangleWorker>(),
            TypeId::of::<MilestoneRequesterWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();
        let milestone_solidifier = node.worker::<MilestoneSolidifierWorker>().unwrap().tx.clone();
        let milestone_cone_updater = node.worker::<MilestoneConeUpdaterWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let key_manager = KeyManager::new(
            config.coordinator.public_key_count,
            config.coordinator.public_key_ranges.into_boxed_slice(),
        );

        let bus = node.resource::<Bus>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(MilestoneValidatorWorkerEvent(message_id)) = receiver.next().await {
                if let Some(meta) = tangle.get_metadata(&message_id) {
                    if meta.flags().is_milestone() {
                        continue;
                    }
                    match validate::<N>(&tangle, &key_manager, message_id).await {
                        Ok(milestone) => {
                            tangle.add_milestone(milestone.index, milestone.message_id);

                            // This is possibly not sufficient as there is no guarantee a milestone has been
                            // solidified before being validated, we then also need
                            // to check when a milestone gets solidified if it's
                            // already vadidated.
                            if meta.flags().is_solid() {
                                bus.dispatch(LatestSolidMilestoneChanged(milestone.clone()));
                                if let Err(e) =
                                    milestone_cone_updater.send(MilestoneConeUpdaterWorkerEvent(milestone.clone()))
                                {
                                    error!("Sending message id to milestone validation failed: {:?}.", e);
                                }
                            }

                            if milestone.index > tangle.get_latest_milestone_index() {
                                bus.dispatch(LatestMilestoneChanged(milestone.clone()));
                            }

                            if requested_milestones.remove(&milestone.index).is_some() {
                                tangle.update_metadata(&milestone.message_id, |meta| {
                                    meta.flags_mut().set_requested(true)
                                });

                                if let Err(e) =
                                    milestone_solidifier.send(MilestoneSolidifierWorkerEvent(milestone.index))
                                {
                                    error!("Sending solidification event failed: {}", e);
                                }
                            }
                        }
                        Err(e) => debug!("Invalid milestone message: {:?}.", e),
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
