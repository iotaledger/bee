// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::Error,
    event::{MilestoneConfirmed, NewOutput, NewSpent},
    merkle_hasher::MerkleHasher,
    metadata::WhiteFlagMetadata,
    model::LedgerIndex,
    storage::{self, StorageBackend},
    white_flag::visit_dfs,
};

use bee_common::{event::Bus, shutdown_stream::ShutdownStream};
use bee_common_pt2::{node::Node, worker::Worker};
use bee_message::{payload::Payload, MessageId};
use bee_tangle::{milestone::MilestoneIndex, MsTangle};

use async_trait::async_trait;
use blake2::Blake2b;
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
        Some(Payload::Milestone(milestone)) => milestone,
        _ => return Err(Error::NoMilestonePayload),
    };

    if milestone.essence().index() != **index + 1 {
        return Err(Error::NonContiguousMilestone(milestone.essence().index(), **index));
    }

    let mut metadata = WhiteFlagMetadata::new(
        MilestoneIndex(milestone.essence().index()),
        milestone.essence().timestamp(),
    );

    if let Err(e) = visit_dfs::<N>(tangle, storage, message_id, &mut metadata).await {
        return Err(e);
    };

    let merkle_proof = MerkleHasher::<Blake2b>::new().digest(&metadata.included_messages);

    if !merkle_proof.eq(&milestone.essence().merkle_proof()) {
        return Err(Error::MerkleProofMismatch(
            hex::encode(merkle_proof),
            hex::encode(milestone.essence().merkle_proof()),
        ));
    }

    if metadata.num_referenced_messages
        != metadata.num_excluded_no_transaction_messages
            + metadata.num_excluded_conflicting_messages
            + metadata.included_messages.len()
    {
        return Err(Error::InvalidMessagesCount(
            metadata.num_referenced_messages,
            metadata.num_excluded_no_transaction_messages,
            metadata.num_excluded_conflicting_messages,
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

    info!(
        "Confirmed milestone {}: referenced {}, no transaction {}, conflicting {}, included {}.",
        milestone.essence().index(),
        metadata.num_referenced_messages,
        metadata.num_excluded_no_transaction_messages,
        metadata.num_excluded_conflicting_messages,
        metadata.included_messages.len()
    );

    bus.dispatch(MilestoneConfirmed {
        index: milestone.essence().index().into(),
        timestamp: milestone.essence().timestamp(),
        referenced_messages: metadata.num_referenced_messages,
        excluded_no_transaction_messages: metadata.num_excluded_no_transaction_messages,
        excluded_conflicting_messages: metadata.num_excluded_conflicting_messages,
        included_messages: metadata.included_messages.len(),
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
