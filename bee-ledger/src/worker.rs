// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    balance::BalanceDiffs,
    dust::DUST_THRESHOLD,
    error::Error,
    event::{MilestoneConfirmed, NewConsumedOutput, NewCreatedOutput},
    model::{Migration, Receipt, TreasuryOutput},
    state::check_ledger_state,
    storage::{self, apply_outputs_diff, create_output, rollback_outputs_diff, store_balance_diffs, StorageBackend},
    white_flag,
    white_flag::{conflict::ConflictReason, merkle_hasher::MerkleHasher, metadata::WhiteFlagMetadata},
};

use bee_message::{
    ledger_index::LedgerIndex,
    milestone::MilestoneIndex,
    output::{self, CreatedOutput, Output, OutputId},
    payload::{transaction::TransactionId, Payload},
    MessageId,
};
use bee_runtime::{event::Bus, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_snapshot::{milestone_diff::MilestoneDiff, SnapshotWorker};
use bee_tangle::{MsTangle, TangleWorker};

use async_trait::async_trait;
use crypto::hashes::blake2b::Blake2b256;
use futures::stream::StreamExt;
use log::{error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, collections::HashMap, convert::TryInto};

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

    let mut metadata = WhiteFlagMetadata::new(MilestoneIndex(milestone.essence().index()));

    let parents = message.parents().to_vec();

    drop(message);

    white_flag::validation::traversal::<N>(tangle, storage, parents, &mut metadata).await?;

    // Account for the milestone itself.
    metadata.num_referenced_messages += 1;
    metadata.excluded_no_transaction_messages.push(message_id);

    let merkle_proof = MerkleHasher::<Blake2b256>::new().digest(&metadata.included_messages);

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

    let migration = if let Some(Payload::Receipt(receipt)) = milestone.essence().receipt() {
        // TODO check that the treasuryTransaction input matches the fetched unspent treasury output ?
        let milestone_id = milestone.id();
        let receipt = Receipt::new(receipt.as_ref().clone(), milestone.essence().index().into());
        let consumed_treasury = storage::fetch_unspent_treasury_output(storage).await?;

        // TODO validate receipt

        // Safe to unwrap since sizes are known to be the same
        let fake_transaction_id = TransactionId::new(milestone_id.as_ref().to_vec().try_into().unwrap());
        // Safe to unwrap since sizes are known to be the same
        let fake_message_id = MessageId::new(milestone_id.as_ref().to_vec().try_into().unwrap());

        for (index, funds) in receipt.inner().funds().iter().enumerate() {
            metadata.created_outputs.insert(
                // Safe to unwrap because indexes are known to be valid at this point.
                OutputId::new(fake_transaction_id, index as u16).unwrap(),
                CreatedOutput::new(fake_message_id, Output::from(funds.output().clone())),
            );
        }

        let created_treasury = TreasuryOutput::new(
            match receipt.inner().transaction() {
                Payload::TreasuryTransaction(treasury) => match treasury.output() {
                    Output::Treasury(output) => output.clone(),
                    _ => return Err(Error::UnsupportedOutputType),
                },
                _ => return Err(Error::UnsupportedPayloadType),
            },
            milestone_id,
        );

        Some(Migration::new(receipt, created_treasury, consumed_treasury))
    } else {
        None
    };

    storage::apply_outputs_diff(
        &*storage,
        metadata.index,
        &metadata.created_outputs,
        &metadata.consumed_outputs,
        &metadata.balance_diffs,
        &migration,
    )
    .await?;

    *index = LedgerIndex(MilestoneIndex(milestone.essence().index()));
    tangle.update_confirmed_milestone_index(milestone.essence().index().into());

    for message_id in metadata.excluded_no_transaction_messages.iter() {
        tangle
            .update_metadata(message_id, |message_metadata| {
                message_metadata.set_conflict(ConflictReason::None as u8);
                message_metadata.confirm(milestone.essence().timestamp());
            })
            .await;
    }

    for (message_id, conflict) in metadata.excluded_conflicting_messages.iter() {
        tangle
            .update_metadata(message_id, |message_metadata| {
                message_metadata.set_conflict(*conflict as u8);
                message_metadata.confirm(milestone.essence().timestamp());
            })
            .await;
    }

    for message_id in metadata.included_messages.iter() {
        tangle
            .update_metadata(message_id, |message_metadata| {
                message_metadata.set_conflict(ConflictReason::None as u8);
                message_metadata.confirm(milestone.essence().timestamp());
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
        created_outputs: metadata.created_outputs.len(),
        consumed_outputs: metadata.consumed_outputs.len(),
    });

    for (_, output) in metadata.created_outputs {
        bus.dispatch(NewCreatedOutput(output));
    }

    for (_, spent) in metadata.consumed_outputs {
        bus.dispatch(NewConsumedOutput(spent));
    }

    Ok(())
}

#[async_trait]
impl<N: Node> Worker<N> for LedgerWorker
where
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Error;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<SnapshotWorker>(), TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let storage = node.storage();
        let bus = node.bus();

        let treasury_output_rx = node.worker::<SnapshotWorker>().unwrap().treasury_output_rx.clone();
        let output_rx = node.worker::<SnapshotWorker>().unwrap().output_rx.clone();
        let full_diff_rx = node.worker::<SnapshotWorker>().unwrap().full_diff_rx.clone();
        let delta_diff_rx = node.worker::<SnapshotWorker>().unwrap().delta_diff_rx.clone();

        // TODO handle Err
        if let Ok((milestone_id, amount)) = treasury_output_rx.recv_async().await {
            storage::store_unspent_treasury_output(
                &*storage,
                &TreasuryOutput::new(output::TreasuryOutput::new(amount)?, milestone_id),
            )
            .await?
        }

        let mut balance_diffs = BalanceDiffs::new();

        while let Ok((output_id, output)) = output_rx.recv_async().await {
            // TODO handle unwrap
            // TODO batch
            create_output(&*storage, &output_id, &output).await.unwrap();
            match output.inner() {
                Output::SignatureLockedSingle(output) => {
                    balance_diffs.amount_add(*output.address(), output.amount());
                    if output.amount() < DUST_THRESHOLD {
                        balance_diffs.dust_output_inc(*output.address());
                    }
                }
                Output::SignatureLockedDustAllowance(output) => {
                    balance_diffs.amount_add(*output.address(), output.amount());
                    balance_diffs.dust_allowance_add(*output.address(), output.amount());
                }
                _ => return Err(Error::UnsupportedOutputType),
            }
        }

        store_balance_diffs(&*storage, &balance_diffs).await?;

        async fn read_diffs<B: StorageBackend>(
            storage: &B,
            diff_rx: flume::Receiver<MilestoneDiff>,
        ) -> Result<(), Error> {
            while let Ok(diff) = diff_rx.recv_async().await {
                let index = diff.milestone().essence().index();
                // Unwrap is fine because we just inserted the ledger index.
                // TODO unwrap
                let ledger_index = *storage::fetch_ledger_index(&*storage).await.unwrap().unwrap();

                let mut balance_diffs = BalanceDiffs::new();

                for (_, output) in diff.created().iter() {
                    match output.inner() {
                        Output::SignatureLockedSingle(output) => {
                            balance_diffs.amount_add(*output.address(), output.amount());
                            if output.amount() < DUST_THRESHOLD {
                                balance_diffs.dust_output_inc(*output.address());
                            }
                        }
                        Output::SignatureLockedDustAllowance(output) => {
                            balance_diffs.amount_add(*output.address(), output.amount());
                            balance_diffs.dust_allowance_add(*output.address(), output.amount());
                        }
                        _ => return Err(Error::UnsupportedOutputType),
                    }
                }

                let mut consumed = HashMap::new();

                for (output_id, (created_output, consumed_output)) in diff.consumed().iter() {
                    match created_output.inner() {
                        Output::SignatureLockedSingle(created_output) => {
                            balance_diffs.amount_sub(*created_output.address(), created_output.amount());
                            if created_output.amount() < DUST_THRESHOLD {
                                balance_diffs.dust_output_dec(*created_output.address());
                            }
                        }
                        Output::SignatureLockedDustAllowance(created_output) => {
                            balance_diffs.amount_sub(*created_output.address(), created_output.amount());
                            balance_diffs.dust_allowance_sub(*created_output.address(), created_output.amount());
                        }
                        _ => return Err(Error::UnsupportedOutputType),
                    }
                    consumed.insert(*output_id, (*consumed_output).clone());
                }

                match index {
                    index if index == ledger_index + 1 => {
                        // TODO unwrap until we merge both crates
                        apply_outputs_diff(
                            &*storage,
                            MilestoneIndex(index),
                            diff.created(),
                            &consumed,
                            &balance_diffs,
                            &None,
                        )
                        .await
                        .unwrap();
                    }
                    index if index == ledger_index => {
                        // TODO unwrap until we merge both crates
                        rollback_outputs_diff(&*storage, MilestoneIndex(index), diff.created(), &consumed)
                            .await
                            .unwrap();
                    }
                    _ => return Err(Error::UnexpectedDiffIndex(index.into())),
                }
            }
            Ok(())
        }

        read_diffs(&*storage, full_diff_rx).await?;
        read_diffs(&*storage, delta_diff_rx).await?;

        check_ledger_state(&*storage).await?;

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

        // Unwrap is fine because we just inserted the ledger index.
        // TODO unwrap
        let mut ledger_index = storage::fetch_ledger_index(&*storage).await.unwrap().unwrap();
        tangle.update_solid_milestone_index(MilestoneIndex(*ledger_index));
        tangle.update_confirmed_milestone_index(MilestoneIndex(*ledger_index));

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(LedgerWorkerEvent(message_id)) = receiver.next().await {
                if let Err(e) = confirm::<N>(&tangle, &storage, &bus, message_id, &mut ledger_index).await {
                    error!("Confirmation error on {}: {}.", message_id, e);
                    panic!("Aborting due to unexpected ledger error.");
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
