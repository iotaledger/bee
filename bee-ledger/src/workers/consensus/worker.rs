// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::{Balance, CreatedOutput, LedgerIndex, Migration, Receipt, TreasuryOutput},
    workers::{
        consensus::{metadata::WhiteFlagMetadata, state::validate_ledger_state, white_flag},
        error::Error,
        event::{MilestoneConfirmed, OutputConsumed, OutputCreated},
        pruning::{
            condition::{should_prune, should_snapshot},
            config::PruningConfig,
            constants::{PRUNING_THRESHOLD, SOLID_ENTRY_POINT_THRESHOLD_FUTURE, SOLID_ENTRY_POINT_THRESHOLD_PAST},
        },
        snapshot::{config::SnapshotConfig, worker::SnapshotWorker},
        storage::{self, StorageBackend},
    },
};

use bee_message::{
    address::Address,
    milestone::MilestoneIndex,
    output::{Output, OutputId},
    payload::{milestone::MilestoneId, receipt::ReceiptPayload, transaction::TransactionId, Payload},
    MessageId,
};
use bee_runtime::{event::Bus, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{ConflictReason, MsTangle, TangleWorker};

use async_trait::async_trait;

use futures::{channel::oneshot, stream::StreamExt};
use log::{error, info, warn};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, convert::TryInto};

/// Event of the consensus worker.
#[allow(clippy::type_complexity)]
pub enum ConsensusWorkerEvent {
    /// Event to confirm a milestone.
    Confirm(MessageId),
    /// Event to fetch the balance of an address.
    Balance(Address, oneshot::Sender<(Result<Option<Balance>, Error>, LedgerIndex)>),
    /// Event to fetch an output.
    Output(
        OutputId,
        oneshot::Sender<(Result<Option<CreatedOutput>, Error>, LedgerIndex)>,
    ),
    /// Event to fetch the outputs of an address.
    Outputs(
        Address,
        oneshot::Sender<(Result<Option<Vec<OutputId>>, Error>, LedgerIndex)>,
    ),
}

/// The consensus worker.
pub struct ConsensusWorker {
    /// Communication channel of the consensus worker.
    pub tx: mpsc::UnboundedSender<ConsensusWorkerEvent>,
}

pub(crate) async fn migration_from_milestone(
    milestone_index: MilestoneIndex,
    milestone_id: MilestoneId,
    receipt: &ReceiptPayload,
    consumed_treasury: TreasuryOutput,
) -> Result<Migration, Error> {
    let receipt = Receipt::new(receipt.clone(), milestone_index);

    receipt.validate(&consumed_treasury)?;

    let created_treasury = TreasuryOutput::new(
        match receipt.inner().transaction() {
            Payload::TreasuryTransaction(treasury) => match treasury.output() {
                Output::Treasury(output) => output.clone(),
                Output::SignatureLockedDustAllowance(_) | Output::SignatureLockedSingle(_) => {
                    return Err(Error::UnsupportedOutputKind(treasury.output().kind()));
                }
            },
            Payload::Milestone(_) | Payload::Indexation(_) | Payload::Receipt(_) | Payload::Transaction(_) => {
                return Err(Error::UnsupportedPayloadKind(receipt.inner().transaction().kind()));
            }
        },
        milestone_id,
    );

    Ok(Migration::new(receipt, consumed_treasury, created_treasury))
}

async fn confirm<N: Node>(
    tangle: &MsTangle<N::Backend>,
    storage: &N::Backend,
    bus: &Bus<'static>,
    message_id: MessageId,
    ledger_index: &mut LedgerIndex,
    receipt_migrated_at: &mut MilestoneIndex,
) -> Result<(), Error>
where
    N::Backend: StorageBackend,
{
    let message = tangle
        .get(&message_id)
        .await
        .ok_or(Error::MilestoneMessageNotFound(message_id))?;

    let milestone = match message.payload() {
        Some(Payload::Milestone(milestone)) => milestone.clone(),
        _ => return Err(Error::NoMilestonePayload),
    };

    if milestone.essence().index() != MilestoneIndex(**ledger_index + 1) {
        return Err(Error::NonContiguousMilestones(
            *milestone.essence().index(),
            **ledger_index,
        ));
    }

    let mut metadata = WhiteFlagMetadata::new(milestone.essence().index());

    white_flag(tangle, storage, message.parents(), &mut metadata).await?;

    if !metadata.merkle_proof.eq(&milestone.essence().merkle_proof()) {
        return Err(Error::MerkleProofMismatch(
            milestone.essence().index(),
            hex::encode(metadata.merkle_proof),
            hex::encode(milestone.essence().merkle_proof()),
        ));
    }

    // Account for the milestone itself.
    metadata.referenced_messages += 1;
    metadata.excluded_no_transaction_messages.push(message_id);

    let migration = if let Some(Payload::Receipt(receipt)) = milestone.essence().receipt() {
        let milestone_id = milestone.id();

        // Safe to unwrap since sizes are known to be the same
        let transaction_id = TransactionId::new(milestone_id.as_ref().to_vec().try_into().unwrap());

        for (index, fund) in receipt.funds().iter().enumerate() {
            metadata.created_outputs.insert(
                OutputId::new(transaction_id, index as u16)?,
                CreatedOutput::new(message_id, Output::from(fund.output().clone())),
            );
            metadata
                .balance_diffs
                .amount_add(*fund.output().address(), fund.output().amount())?;
        }

        if receipt.migrated_at() < *receipt_migrated_at {
            return Err(Error::DecreasingReceiptMigratedAtIndex(
                receipt.migrated_at(),
                *receipt_migrated_at,
            ));
        } else {
            *receipt_migrated_at = receipt.migrated_at();
        }

        if receipt.last() {
            *receipt_migrated_at = *receipt_migrated_at + 1;
        }

        Some(
            migration_from_milestone(
                milestone.essence().index(),
                milestone_id,
                receipt,
                storage::fetch_unspent_treasury_output(storage).await?,
            )
            .await?,
        )
    } else {
        None
    };

    storage::apply_milestone(
        &*storage,
        metadata.index,
        &metadata.created_outputs,
        &metadata.consumed_outputs,
        &metadata.balance_diffs,
        &migration,
    )
    .await?;

    *ledger_index = LedgerIndex(milestone.essence().index());
    tangle.update_confirmed_milestone_index(milestone.essence().index());

    for message_id in metadata.excluded_no_transaction_messages.iter() {
        tangle
            .update_metadata(message_id, |message_metadata| {
                message_metadata.set_conflict(ConflictReason::None);
                message_metadata.reference(milestone.essence().timestamp());
            })
            .await;
    }

    for (message_id, conflict) in metadata.excluded_conflicting_messages.iter() {
        tangle
            .update_metadata(message_id, |message_metadata| {
                message_metadata.set_conflict(*conflict);
                message_metadata.reference(milestone.essence().timestamp());
            })
            .await;
    }

    for message_id in metadata.included_messages.iter() {
        tangle
            .update_metadata(message_id, |message_metadata| {
                message_metadata.set_conflict(ConflictReason::None);
                message_metadata.reference(milestone.essence().timestamp());
            })
            .await;
    }

    info!(
        "Confirmed milestone {}: referenced {}, no transaction {}, conflicting {}, included {}, consumed {}, created {}, receipt {}.",
        milestone.essence().index(),
        metadata.referenced_messages,
        metadata.excluded_no_transaction_messages.len(),
        metadata.excluded_conflicting_messages.len(),
        metadata.included_messages.len(),
        metadata.consumed_outputs.len(),
        metadata.created_outputs.len(),
        milestone.essence().receipt().is_some()
    );

    bus.dispatch(MilestoneConfirmed {
        message_id,
        index: milestone.essence().index(),
        timestamp: milestone.essence().timestamp(),
        referenced_messages: metadata.referenced_messages,
        excluded_no_transaction_messages: metadata.excluded_no_transaction_messages,
        excluded_conflicting_messages: metadata.excluded_conflicting_messages,
        included_messages: metadata.included_messages,
        consumed_outputs: metadata.consumed_outputs.len(),
        created_outputs: metadata.created_outputs.len(),
        receipt: migration.is_some(),
    });

    for (_, created_output) in metadata.created_outputs {
        bus.dispatch(OutputCreated { output: created_output });
    }

    for (_, (_, consumed_output)) in metadata.consumed_outputs {
        bus.dispatch(OutputConsumed {
            output: consumed_output,
        });
    }

    Ok(())
}

#[async_trait]
impl<N: Node> Worker<N> for ConsensusWorker
where
    N::Backend: StorageBackend,
{
    type Config = (SnapshotConfig, PruningConfig);
    type Error = Error;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>(), TypeId::of::<SnapshotWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (snapshot_config, pruning_config) = config;
        let (tx, rx) = mpsc::unbounded_channel();
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let storage = node.storage();
        let bus = node.bus();

        validate_ledger_state(&*storage).await?;

        let depth = if snapshot_config.depth() < SOLID_ENTRY_POINT_THRESHOLD_FUTURE {
            warn!(
                "Configuration value for \"depth\" is too low ({}), value changed to {}.",
                snapshot_config.depth(),
                SOLID_ENTRY_POINT_THRESHOLD_FUTURE
            );
            SOLID_ENTRY_POINT_THRESHOLD_FUTURE
        } else {
            snapshot_config.depth()
        };
        let delay_min = snapshot_config.depth() + SOLID_ENTRY_POINT_THRESHOLD_PAST + PRUNING_THRESHOLD + 1;
        let delay = if pruning_config.delay() < delay_min {
            warn!(
                "Configuration value for \"delay\" is too low ({}), value changed to {}.",
                pruning_config.delay(),
                delay_min
            );
            delay_min
        } else {
            pruning_config.delay()
        };

        // Unwrap is fine because ledger index was already in storage or just added by the snapshot worker.
        let mut ledger_index = storage::fetch_ledger_index(&*storage).await?.unwrap();
        let mut receipt_migrated_at = MilestoneIndex(0);

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(event) = receiver.next().await {
                match event {
                    ConsensusWorkerEvent::Confirm(message_id) => {
                        if let Err(e) = confirm::<N>(
                            &tangle,
                            &storage,
                            &bus,
                            message_id,
                            &mut ledger_index,
                            &mut receipt_migrated_at,
                        )
                        .await
                        {
                            error!("Confirmation error on {}: {}.", message_id, e);
                            panic!("Aborting due to unexpected ledger error.");
                        }

                        if !tangle.is_confirmed() {
                            continue;
                        }

                        if should_snapshot(&tangle, MilestoneIndex(*ledger_index), depth, &snapshot_config) {
                            // TODO
                            // if let Err(e) = snapshot(snapshot_config.path(), event.index - depth) {
                            //     error!("Failed to create snapshot: {:?}.", e);
                            // }
                        }

                        if should_prune(&tangle, MilestoneIndex(*ledger_index), delay, &pruning_config) {
                            // TODO
                            // if let Err(e) = prune_database(&tangle, MilestoneIndex(*event.index - delay)) {
                            //     error!("Failed to prune database: {:?}.", e);
                            // }
                        }
                    }
                    ConsensusWorkerEvent::Balance(address, sender) => {
                        if let Err(e) = sender.send((storage::fetch_balance(&*storage, &address).await, ledger_index)) {
                            error!("Error while sending balance: {:?}", e);
                        }
                    }
                    ConsensusWorkerEvent::Output(output_id, sender) => {
                        if let Err(e) = sender.send((storage::fetch_output(&*storage, &output_id).await, ledger_index))
                        {
                            error!("Error while sending output: {:?}", e);
                        }
                    }
                    ConsensusWorkerEvent::Outputs(address, sender) => match address {
                        Address::Ed25519(address) => {
                            if let Err(e) = sender.send((
                                storage::fetch_outputs_for_ed25519_address(&*storage, &address).await,
                                ledger_index,
                            )) {
                                error!("Error while sending output: {:?}", e);
                            }
                        }
                    },
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
