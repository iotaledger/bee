// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    conflict::ConflictReason,
    error::Error,
    event::{MilestoneConfirmed, NewOutput, NewSpent},
    merkle_hasher::MerkleHasher,
    metadata::WhiteFlagMetadata,
    storage::{self, StorageBackend},
    white_flag,
};

use bee_message::{ledger_index::LedgerIndex, milestone::MilestoneIndex, payload::Payload, MessageId};
use bee_runtime::{event::Bus, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::MsTangle;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{error, info};
use tokio::sync::mpsc;

use std::convert::Infallible;

pub struct LedgerWorkerEvent(pub MessageId);

pub struct LedgerWorker {
    pub tx: mpsc::UnboundedSender<LedgerWorkerEvent>,
}

async fn confirm<N: Node>(
    tangle: &MsTangle<N::Backend>,
    storage: &N::Backend,
    bus: &Bus<'static>,
    message_id: MessageId,
    index: &mut LedgerIndex,
) -> Result<(), Error>
where
    N::Backend: StorageBackend,
{
    let message = tangle.get(&message_id).await.ok_or(Error::MilestoneMessageNotFound)?;

    let milestone = match message.payload() {
        Some(Payload::Milestone(milestone)) => milestone.clone(),
        _ => return Err(Error::NoMilestonePayload),
    };

    if milestone.essence().index() != **index + 1 {
        return Err(Error::NonContiguousMilestone(milestone.essence().index(), **index));
    }

    let mut metadata = WhiteFlagMetadata::new(
        MilestoneIndex(milestone.essence().index()),
        milestone.essence().timestamp(),
    );

    let (parent1, parent2) = (*message.parent1(), *message.parent2());

    drop(message);

    if let Err(e) = white_flag::traversal::<N>(tangle, storage, vec![parent1, parent2], &mut metadata).await {
        return Err(e);
    };

    // Account for the milestone itself.
    metadata.num_referenced_messages += 1;
    metadata.excluded_no_transaction_messages.push(message_id);

    let merkle_proof = MerkleHasher::new().digest(&metadata.included_messages);

    if !merkle_proof.eq(&milestone.essence().merkle_proof()) {
        return Err(Error::MerkleProofMismatch(
            hex::encode(merkle_proof),
            hex::encode(milestone.essence().merkle_proof()),
        ));
    }

    if metadata.num_referenced_messages
        != metadata.excluded_no_transaction_messages.len()
            + metadata.excluded_conflicting_messages.len()
            + metadata.included_messages.len()
    {
        return Err(Error::InvalidMessagesCount(
            metadata.num_referenced_messages,
            metadata.excluded_no_transaction_messages.len(),
            metadata.excluded_conflicting_messages.len(),
            metadata.included_messages.len(),
        ));
    }

    storage::apply_diff(
        &*storage,
        metadata.index,
        &metadata.spent_outputs,
        &metadata.created_outputs,
    )
    .await?;

    *index = LedgerIndex(MilestoneIndex(milestone.essence().index()));

    for message_id in metadata.excluded_no_transaction_messages.iter() {
        tangle
            .update_metadata(message_id, |message_metadata| {
                message_metadata.set_conflict(ConflictReason::None as u8);
                message_metadata.set_milestone_index(metadata.index);
                message_metadata.confirm(metadata.timestamp);
            })
            .await;
    }

    for (message_id, conflict) in metadata.excluded_conflicting_messages.iter() {
        tangle
            .update_metadata(message_id, |message_metadata| {
                message_metadata.set_conflict(*conflict as u8);
                message_metadata.set_milestone_index(metadata.index);
                message_metadata.confirm(metadata.timestamp);
            })
            .await;
    }

    for message_id in metadata.included_messages.iter() {
        tangle
            .update_metadata(message_id, |message_metadata| {
                message_metadata.set_conflict(ConflictReason::None as u8);
                message_metadata.set_milestone_index(metadata.index);
                message_metadata.confirm(metadata.timestamp);
            })
            .await;
    }

    info!(
        "Confirmed milestone {}: referenced {}, no transaction {}, conflicting {}, included {}.",
        milestone.essence().index(),
        metadata.num_referenced_messages,
        metadata.excluded_no_transaction_messages.len(),
        metadata.excluded_conflicting_messages.len(),
        metadata.included_messages.len()
    );

    bus.dispatch(MilestoneConfirmed {
        id: message_id,
        index: milestone.essence().index().into(),
        timestamp: milestone.essence().timestamp(),
        referenced_messages: metadata.num_referenced_messages,
        excluded_no_transaction_messages: metadata.excluded_no_transaction_messages,
        excluded_conflicting_messages: metadata.excluded_conflicting_messages,
        included_messages: metadata.included_messages,
        spent_outputs: metadata.spent_outputs.len(),
        created_outputs: metadata.created_outputs.len(),
    });

    for (_, spent) in metadata.spent_outputs {
        bus.dispatch(NewSpent(spent));
    }

    for (_, output) in metadata.created_outputs {
        bus.dispatch(NewOutput(output));
    }

    Ok(())
}

#[async_trait]
impl<N: Node> Worker<N> for LedgerWorker
where
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Infallible;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let storage = node.storage();
        let bus = node.bus();

        // bus.add_listener::<Self, LatestSolidMilestoneChanged, _>(move |event| {
        //     if let Err(e) = tx.send(*event.milestone.message_id()) {
        //         warn!(
        //             "Sending solid milestone {} {} to confirmation failed: {:?}.",
        //             *event.index,
        //             event.milestone.message_id(),
        //             e
        //         )
        //     }
        // });

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);
            // Unwrap is fine because we just inserted the ledger index.
            // TODO unwrap
            let mut index = storage::fetch_ledger_index(&*storage).await.unwrap().unwrap();

            while let Some(LedgerWorkerEvent(message_id)) = receiver.next().await {
                if let Err(e) = confirm::<N>(&tangle, &storage, &bus, message_id, &mut index).await {
                    error!("Confirmation error on {}: {}.", message_id, e);
                    panic!("Aborting due to unexpected ledger error.");
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
