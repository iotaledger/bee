// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::Error, event::MilestoneConfirmed, merkle_hasher::MerkleHasher, metadata::WhiteFlagMetadata,
    storage::Backend, white_flag::visit_dfs,
};

use bee_common::{
    event::Bus,
    node::{Node, ResHandle},
    shutdown_stream::ShutdownStream,
    worker::Worker,
};
use bee_message::{payload::Payload, MessageId};
use bee_protocol::{
    config::ProtocolCoordinatorConfig, event::LatestSolidMilestoneChanged, tangle::MsTangle, MilestoneIndex,
    StorageWorker, TangleWorker,
};
use bee_storage::access::BatchBuilder;

use async_trait::async_trait;
use blake2::Blake2b;
use futures::stream::StreamExt;
use log::{error, info, warn};

use std::{any::TypeId, convert::Infallible, sync::Arc};

pub(crate) struct LedgerWorker {}

async fn confirm<N: Node>(
    tangle: &MsTangle<N::Backend>,
    storage: &ResHandle<N::Backend>,
    message_id: MessageId,
    index: &mut MilestoneIndex,
    _coo_config: &ProtocolCoordinatorConfig,
    bus: &Arc<Bus<'static>>,
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
        // TODO useful ?
        milestone.essence().timestamp(),
    );

    let batch = N::Backend::batch_begin();

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

    // TODO unwrap
    storage.batch_commit(batch, true).await.unwrap();

    // TODO update meta only when sure everything is fine

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
    type Config = (MilestoneIndex, ProtocolCoordinatorConfig, Arc<Bus<'static>>);
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>(), TypeId::of::<StorageWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let storage = node.storage();
        let bus = node.resource::<Bus>();

        bus.add_listener::<(), LatestSolidMilestoneChanged, _>(move |milestone| {
            if let Err(e) = tx.send(*milestone.0.message_id()) {
                warn!(
                    "Sending solid milestone {} {} to confirmation failed: {:?}.",
                    *milestone.0.index(),
                    milestone.0.message_id(),
                    e
                )
            }
        });

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            let mut index = config.0;
            let coo_config = config.1;
            let bus = config.2;

            while let Some(message_id) = receiver.next().await {
                if let Err(e) = confirm::<N>(&tangle, &storage, message_id, &mut index, &coo_config, &bus).await {
                    error!("Confirmation error on {}: {}.", message_id, e);
                    panic!("Aborting due to unexpected ledger error.");
                }
            }

            info!("Stopped.");
        });

        Ok(Self {})
    }
}
