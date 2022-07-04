// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::any::TypeId;

use async_trait::async_trait;
use bee_block::{
    output::{unlock_condition::AddressUnlockCondition, BasicOutput, Output, OutputId},
    payload::{
        milestone::{MilestoneId, MilestoneIndex, ReceiptMilestoneOption},
        transaction::TransactionId,
        Payload,
    },
    semantic::ConflictReason,
    BlockId,
};
use bee_runtime::{event::Bus, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{Tangle, TangleWorker};
use futures::{channel::oneshot, stream::StreamExt};
use log::{debug, error, info, warn};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    types::{CreatedOutput, LedgerIndex, Migration, Receipt, TreasuryOutput},
    workers::{
        consensus::{metadata::WhiteFlagMetadata, state::validate_ledger_state, white_flag},
        error::Error,
        event::{BlockReferenced, LedgerUpdated, MilestoneConfirmed, OutputConsumed, OutputCreated, ReceiptCreated},
        pruning::{condition::should_prune, config::PruningConfig, prune},
        snapshot::{condition::should_snapshot, config::SnapshotConfig, worker::SnapshotWorker},
        storage::{self, StorageBackend},
    },
};

pub(crate) const EXTRA_SNAPSHOT_DEPTH: u32 = 5;
pub(crate) const EXTRA_PRUNING_DEPTH: u32 = 5;

/// Commands of the consensus worker.
#[allow(clippy::type_complexity)]
pub enum ConsensusWorkerCommand {
    /// Command to confirm a milestone.
    ConfirmMilestone(BlockId),
    /// Command to fetch an output.
    FetchOutput(
        OutputId,
        oneshot::Sender<(Result<Option<CreatedOutput>, Error>, LedgerIndex)>,
    ),
}

/// The consensus worker.
pub struct ConsensusWorker {
    /// Communication channel of the consensus worker.
    pub tx: mpsc::UnboundedSender<ConsensusWorkerCommand>,
}

pub(crate) fn migration_from_milestone(
    milestone_index: MilestoneIndex,
    milestone_id: MilestoneId,
    receipt: &ReceiptMilestoneOption,
    consumed_treasury: TreasuryOutput,
) -> Result<Migration, Error> {
    let receipt = Receipt::new(receipt.clone(), milestone_index);

    receipt.validate(&consumed_treasury)?;

    let created_treasury = TreasuryOutput::new(receipt.inner().transaction().output().clone(), milestone_id);

    Ok(Migration::new(receipt, consumed_treasury, created_treasury))
}

async fn confirm<N: Node>(
    tangle: &Tangle<N::Backend>,
    storage: &N::Backend,
    bus: &Bus<'static>,
    block_id: BlockId,
    ledger_index: &mut LedgerIndex,
    receipt_migrated_at: &mut MilestoneIndex,
) -> Result<(), Error>
where
    N::Backend: StorageBackend,
{
    let block = tangle.get(&block_id).ok_or(Error::MilestoneBlockNotFound(block_id))?;

    let milestone = match block.payload() {
        Some(Payload::Milestone(milestone)) => milestone,
        _ => return Err(Error::NoMilestonePayload),
    };

    if milestone.essence().index() != MilestoneIndex(**ledger_index + 1) {
        return Err(Error::NonContiguousMilestones(
            *milestone.essence().index(),
            **ledger_index,
        ));
    }

    let mut metadata = WhiteFlagMetadata::new(
        milestone.essence().index(),
        milestone.essence().timestamp(),
        Some(*milestone.essence().previous_milestone_id()),
    );

    white_flag(tangle, storage, block.parents(), &mut metadata).await?;

    if metadata.inclusion_merkle_root != *milestone.essence().inclusion_merkle_root() {
        return Err(Error::InclusionMerkleRootMismatch(
            milestone.essence().index(),
            metadata.inclusion_merkle_root,
            *milestone.essence().inclusion_merkle_root(),
        ));
    }

    if metadata.applied_merkle_root != *milestone.essence().applied_merkle_root() {
        return Err(Error::AppliedMerkleRootMismatch(
            milestone.essence().index(),
            metadata.applied_merkle_root,
            *milestone.essence().applied_merkle_root(),
        ));
    }

    let migration = if let Some(receipt) = milestone.essence().options().receipt() {
        let milestone_id = milestone.id();
        let transaction_id = TransactionId::from(milestone_id);

        for (index, fund) in receipt.funds().iter().enumerate() {
            metadata.created_outputs.insert(
                OutputId::new(transaction_id, index as u16)?,
                CreatedOutput::new(
                    block_id,
                    milestone.essence().index(),
                    milestone.essence().timestamp() as u32,
                    Output::from(
                        BasicOutput::build_with_amount(fund.amount())
                            // PANIC: funds are already syntactically verified as part of the receipt validation.
                            .unwrap()
                            .add_unlock_condition(AddressUnlockCondition::new(*fund.address()).into())
                            .finish()
                            // PANIC: these parameters are certified fine.
                            .unwrap(),
                    ),
                ),
            );
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

        Some(migration_from_milestone(
            milestone.essence().index(),
            milestone_id,
            receipt,
            storage::fetch_unspent_treasury_output(storage)?,
        )?)
    } else {
        None
    };

    storage::apply_milestone(
        storage,
        metadata.milestone_index,
        &metadata.created_outputs,
        &metadata.consumed_outputs,
        &migration,
    )?;

    *ledger_index = LedgerIndex(milestone.essence().index());
    tangle.update_confirmed_milestone_index(milestone.essence().index());

    for block_id in metadata.excluded_no_transaction_blocks.iter() {
        tangle.update_metadata(block_id, |block_metadata| {
            block_metadata.set_conflict(ConflictReason::None);
            block_metadata.reference(milestone.essence().timestamp());
        });
        bus.dispatch(BlockReferenced { block_id: *block_id });
    }

    for (block_id, conflict) in metadata.excluded_conflicting_blocks.iter() {
        tangle.update_metadata(block_id, |block_metadata| {
            block_metadata.set_conflict(*conflict);
            block_metadata.reference(milestone.essence().timestamp());
        });
        bus.dispatch(BlockReferenced { block_id: *block_id });
    }

    for block_id in metadata.included_blocks.iter() {
        tangle.update_metadata(block_id, |block_metadata| {
            block_metadata.set_conflict(ConflictReason::None);
            block_metadata.reference(milestone.essence().timestamp());
        });
        bus.dispatch(BlockReferenced { block_id: *block_id });
    }

    info!(
        "Confirmed milestone {}: referenced {}, no transaction {}, conflicting {}, included {}, consumed {}, created {}, receipt {}.",
        milestone.essence().index(),
        metadata.referenced_blocks.len(),
        metadata.excluded_no_transaction_blocks.len(),
        metadata.excluded_conflicting_blocks.len(),
        metadata.included_blocks.len(),
        metadata.consumed_outputs.len(),
        metadata.created_outputs.len(),
        milestone.essence().options().receipt().is_some()
    );

    bus.dispatch(MilestoneConfirmed {
        block_id,
        index: milestone.essence().index(),
        timestamp: milestone.essence().timestamp(),
        referenced_blocks: metadata.referenced_blocks.len(),
        excluded_no_transaction_blocks: metadata.excluded_no_transaction_blocks,
        excluded_conflicting_blocks: metadata.excluded_conflicting_blocks,
        included_blocks: metadata.included_blocks,
        consumed_outputs: metadata.consumed_outputs.len(),
        created_outputs: metadata.created_outputs.len(),
        receipt: migration.is_some(),
    });

    for (output_id, created_output) in metadata.created_outputs.iter() {
        bus.dispatch(OutputCreated {
            output_id: *output_id,
            output: created_output.clone(),
        });
    }

    for (output_id, (created_output, _consumed_output)) in metadata.consumed_outputs.iter() {
        bus.dispatch(OutputConsumed {
            block_id: *created_output.block_id(),
            output_id: *output_id,
            output: created_output.inner().clone(),
        });
    }

    bus.dispatch(LedgerUpdated {
        milestone_index: milestone.essence().index(),
        created_outputs: metadata.created_outputs,
        consumed_outputs: metadata.consumed_outputs,
    });

    if let Some(migration) = migration {
        bus.dispatch(ReceiptCreated(migration.into_receipt()));
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
        let tangle = node.resource::<Tangle<N::Backend>>();
        let storage = node.storage();
        let bus = node.bus();

        validate_ledger_state(&*storage)?;

        let bmd = tangle.config().below_max_depth();

        let snapshot_depth_min = bmd + EXTRA_SNAPSHOT_DEPTH;
        let snapshot_depth = if snapshot_config.depth() < snapshot_depth_min {
            warn!(
                "Configuration value for \"snapshot.depth\" is too low ({}), value changed to {}.",
                snapshot_config.depth(),
                snapshot_depth_min
            );
            snapshot_depth_min
        } else {
            snapshot_config.depth()
        };

        let snapshot_pruning_delta = bmd + EXTRA_PRUNING_DEPTH;
        let pruning_delay_min = snapshot_depth + snapshot_pruning_delta;
        let pruning_delay = if pruning_config.delay() < pruning_delay_min {
            warn!(
                "Configuration value for \"pruning.delay\" is too low ({}), value changed to {}.",
                pruning_config.delay(),
                pruning_delay_min
            );
            pruning_delay_min
        } else {
            pruning_config.delay()
        };

        // Unwrap is fine because ledger index was already in storage or just added by the snapshot worker.
        let mut ledger_index = storage::fetch_ledger_index(&*storage)?.unwrap();
        let mut receipt_migrated_at = MilestoneIndex(0);

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(event) = receiver.next().await {
                match event {
                    ConsensusWorkerCommand::ConfirmMilestone(block_id) => {
                        if let Err(e) = confirm::<N>(
                            &tangle,
                            &storage,
                            &bus,
                            block_id,
                            &mut ledger_index,
                            &mut receipt_migrated_at,
                        )
                        .await
                        {
                            error!("Confirmation error on {}: {}.", block_id, e);
                            panic!("Aborting due to unexpected ledger error.");
                        }

                        if !tangle.is_confirmed() {
                            continue;
                        }

                        match should_snapshot(&tangle, ledger_index, snapshot_depth, &snapshot_config) {
                            Ok(()) => {
                                // TODO
                                // if let Err(e) = snapshot(snapshot_config.path(), event.index - snapshot_depth) {
                                //     error!("Failed to create snapshot: {:?}.", e);
                                // }
                            }
                            Err(reason) => {
                                debug!("Snapshotting skipped: {:?}", reason);
                            }
                        }

                        match should_prune(&tangle, ledger_index, pruning_delay, &pruning_config) {
                            Ok((start_index, target_index)) => {
                                if let Err(e) =
                                    prune::prune(&tangle, &storage, &bus, start_index, target_index, &pruning_config)
                                        .await
                                {
                                    error!("Pruning failed: {:?}.", e);
                                }
                            }
                            Err(reason) => {
                                debug!("Pruning skipped: {:?}", reason);
                            }
                        }
                    }
                    ConsensusWorkerCommand::FetchOutput(output_id, sender) => {
                        if let Err(e) = sender.send((storage::fetch_output(&*storage, &output_id), ledger_index)) {
                            error!("Error while sending output: {:?}", e);
                        }
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
