// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::Error,
    event::MilestoneConfirmed,
    merkle_hasher::MerkleHasher,
    metadata::WhiteFlagMetadata,
    storage::{self, Backend},
    white_flag::visit_dfs,
};

use bee_common::{event::Bus, shutdown_stream::ShutdownStream};
use bee_common_pt2::{node::Node, worker::Worker};
use bee_message::{payload::Payload, MessageId};
use bee_protocol::{
    event::LatestSolidMilestoneChanged, tangle::MsTangle, MetricsWorker, MilestoneIndex, ProtocolMetrics, TangleWorker,
};

use async_trait::async_trait;
use blake2::Blake2b;
use futures::stream::StreamExt;
use log::{error, info, warn};

use std::{any::TypeId, convert::Infallible, ops::Deref};

pub(crate) struct LedgerWorker {}

async fn confirm<N: Node>(
    tangle: &MsTangle<N::Backend>,
    storage: &N::Backend,
    message_id: MessageId,
    index: &mut MilestoneIndex,
    bus: &Bus<'static>,
) -> Result<(), Error>
where
    N::Backend: Backend,
{
    let message = tangle.get(&message_id).await.ok_or(Error::MilestoneMessageNotFound)?;

    let milestone = match message.payload() {
        Some(Payload::Milestone(milestone)) => milestone,
        _ => return Err(Error::NoMilestonePayload),
    };

    if milestone.essence().index() != index.0 + 1 {
        return Err(Error::NonContiguousMilestone(milestone.essence().index(), index.0));
    }

    let mut metadata = WhiteFlagMetadata::new(
        MilestoneIndex(milestone.essence().index()),
        milestone.essence().timestamp(),
    );

    if let Err(e) = visit_dfs::<N>(tangle, storage, message_id, &mut metadata).await {
        return Err(e);
    };

    let merkle_proof = MerkleHasher::<Blake2b>::new().digest(&metadata.messages_included);

    if !merkle_proof.eq(&milestone.essence().merkle_proof()) {
        return Err(Error::MerkleProofMismatch(
            hex::encode(merkle_proof),
            hex::encode(milestone.essence().merkle_proof()),
        ));
    }

    if metadata.num_messages_referenced
        != metadata.num_messages_excluded_no_transaction
            + metadata.num_messages_excluded_conflicting
            + metadata.messages_included.len()
    {
        return Err(Error::InvalidMessagesCount(
            metadata.num_messages_referenced,
            metadata.num_messages_excluded_no_transaction,
            metadata.num_messages_excluded_conflicting,
            metadata.messages_included.len(),
        ));
    }

    storage::apply_metadata(storage.deref(), &metadata).await?;

    *index = MilestoneIndex(milestone.essence().index());

    info!(
        "Confirmed milestone {}: referenced {}, zero value {}, conflicting {}, included {}.",
        milestone.essence().index(),
        metadata.num_messages_referenced,
        metadata.num_messages_excluded_no_transaction,
        metadata.num_messages_excluded_conflicting,
        metadata.messages_included.len()
    );

    bus.dispatch(MilestoneConfirmed {
        index: milestone.essence().index().into(),
        timestamp: milestone.essence().timestamp(),
        messages_referenced: metadata.num_messages_referenced,
        messages_excluded_no_transaction: metadata.num_messages_excluded_no_transaction,
        messages_excluded_conflicting: metadata.num_messages_excluded_conflicting,
        messages_included: metadata.messages_included.len(),
    });

    Ok(())
}

#[async_trait]
impl<N: Node> Worker<N> for LedgerWorker
where
    N::Backend: Backend,
{
    type Config = MilestoneIndex;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>(), TypeId::of::<MetricsWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let _metrics = node.resource::<ProtocolMetrics>();
        let storage = node.storage();
        let bus = node.bus();

        bus.add_listener::<(), LatestSolidMilestoneChanged, _>(move |event| {
            if let Err(e) = tx.send(*event.milestone.message_id()) {
                warn!(
                    "Sending solid milestone {} {} to confirmation failed: {:?}.",
                    *event.index,
                    event.milestone.message_id(),
                    e
                )
            }
        });

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());
            let mut index = config;

            while let Some(message_id) = receiver.next().await {
                if let Err(e) = confirm::<N>(&tangle, &storage, message_id, &mut index, &bus).await {
                    error!("Confirmation error on {}: {}.", message_id, e);
                    panic!("Aborting due to unexpected ledger error.");
                }
            }

            info!("Stopped.");
        });

        Ok(Self {})
    }
}
