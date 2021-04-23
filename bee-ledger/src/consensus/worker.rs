// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    consensus::{
        error::Error,
        event::{MilestoneConfirmed, OutputConsumed, OutputCreated},
        metadata::WhiteFlagMetadata,
        state::check_ledger_state,
        storage::{self, StorageBackend},
        white_flag,
    },
    pruning::{
        condition::{should_prune, should_snapshot},
        config::PruningConfig,
        constants::{PRUNING_THRESHOLD, SOLID_ENTRY_POINT_THRESHOLD_FUTURE, SOLID_ENTRY_POINT_THRESHOLD_PAST},
    },
    snapshot::{config::SnapshotConfig, error::Error as SnapshotError, import::import_snapshots},
    types::{CreatedOutput, LedgerIndex, Migration, Receipt, TreasuryOutput},
};

use bee_ledger_types::types::ConflictReason;
use bee_message::{
    milestone::MilestoneIndex,
    output::{Output, OutputId},
    payload::{milestone::MilestoneId, receipt::ReceiptPayload, transaction::TransactionId, Payload},
    MessageId,
};
use bee_runtime::{event::Bus, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_storage::{access::AsStream, backend::StorageBackend as _, health::StorageHealth};
use bee_tangle::{solid_entry_point::SolidEntryPoint, MsTangle, TangleWorker};

use async_trait::async_trait;

use chrono::{offset::TimeZone, Utc};
use futures::stream::StreamExt;
use log::{error, info, warn};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, convert::TryInto};

pub struct LedgerWorkerEvent(pub MessageId);

pub struct LedgerWorker {
    pub tx: mpsc::UnboundedSender<LedgerWorkerEvent>,
}

pub(crate) async fn migration_from_milestone(
    milestone_index: MilestoneIndex,
    milestone_id: MilestoneId,
    receipt: &ReceiptPayload,
    consumed_treasury: TreasuryOutput,
) -> Result<Migration, Error> {
    let receipt = Receipt::new(receipt.clone(), milestone_index);

    // TODO check that the treasuryTransaction input matches the fetched unspent treasury output ?
    receipt.validate(&consumed_treasury)?;

    let created_treasury = TreasuryOutput::new(
        match receipt.inner().transaction() {
            Payload::TreasuryTransaction(treasury) => match treasury.output() {
                Output::Treasury(output) => output.clone(),
                output => return Err(Error::UnsupportedOutputKind(output.kind())),
            },
            payload => return Err(Error::UnsupportedPayloadKind(payload.kind())),
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

    if milestone.essence().index() != MilestoneIndex(**index + 1) {
        return Err(Error::NonContiguousMilestone(*milestone.essence().index(), **index));
    }

    let mut metadata = WhiteFlagMetadata::new(milestone.essence().index());

    let parents = message.parents().iter().copied().collect();

    drop(message);

    white_flag::<N::Backend>(tangle, storage, parents, &mut metadata).await?;

    if !metadata.merkle_proof.eq(&milestone.essence().merkle_proof()) {
        return Err(Error::MerkleProofMismatch(
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

        for (index, funds) in receipt.funds().iter().enumerate() {
            metadata.created_outputs.insert(
                // Safe to unwrap because indexes are known to be valid at this point.
                OutputId::new(transaction_id, index as u16).unwrap(),
                CreatedOutput::new(message_id, Output::from(funds.output().clone())),
            );
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

    *index = LedgerIndex(milestone.essence().index());
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
        id: message_id,
        index: milestone.essence().index(),
        timestamp: milestone.essence().timestamp(),
        referenced_messages: metadata.referenced_messages,
        excluded_no_transaction_messages: metadata.excluded_no_transaction_messages,
        excluded_conflicting_messages: metadata.excluded_conflicting_messages,
        included_messages: metadata.included_messages,
        consumed_outputs: metadata.consumed_outputs.len(),
        created_outputs: metadata.created_outputs.len(),
    });

    for (_, output) in metadata.created_outputs {
        bus.dispatch(OutputCreated(output));
    }

    for (_, spent) in metadata.consumed_outputs {
        bus.dispatch(OutputConsumed(spent));
    }

    Ok(())
}

#[async_trait]
impl<N: Node> Worker<N> for LedgerWorker
where
    N::Backend: StorageBackend,
{
    type Config = (u64, SnapshotConfig, PruningConfig);
    type Error = Error;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (network_id, snapshot_config, pruning_config) = config;

        let (tx, rx) = mpsc::unbounded_channel();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let storage = node.storage();
        let bus = node.bus();

        match storage::fetch_snapshot_info(&*storage).await? {
            None => {
                if let Err(e) = import_snapshots(&*storage, network_id, &snapshot_config).await {
                    (*storage)
                        .set_health(StorageHealth::Corrupted)
                        .await
                        .map_err(|e| Error::Storage(Box::new(e)))?;
                    Err(e)?;
                }
            }
            Some(info) => {
                if info.network_id() != network_id {
                    return Err(Error::Snapshot(SnapshotError::NetworkIdMismatch(
                        info.network_id(),
                        network_id,
                    )));
                }

                info!(
                    "Loaded snapshot from {} with snapshot index {}, entry point index {} and pruning index {}.",
                    Utc.timestamp(info.timestamp() as i64, 0).format("%d-%m-%Y %H:%M:%S"),
                    *info.snapshot_index(),
                    *info.entry_point_index(),
                    *info.pruning_index(),
                );
            }
        }

        check_ledger_state(&*storage).await?;

        {
            let mut solid_entry_points = AsStream::<SolidEntryPoint, MilestoneIndex>::stream(&*storage)
                .await
                .map_err(|e| Error::Storage(Box::new(e)))?;

            while let Some((sep, index)) = solid_entry_points.next().await {
                tangle.add_solid_entry_point(sep, index).await;
            }
        }

        // Unwrap is fine because we just inserted the ledger index.
        // TODO unwrap
        let mut ledger_index = storage::fetch_ledger_index(&*storage).await.unwrap().unwrap();
        let snapshot_info = storage::fetch_snapshot_info(&*storage).await?.unwrap();

        tangle.update_snapshot_index(snapshot_info.snapshot_index());
        tangle.update_pruning_index(snapshot_info.pruning_index());
        tangle.update_solid_milestone_index(MilestoneIndex(*ledger_index));
        tangle.update_confirmed_milestone_index(MilestoneIndex(*ledger_index));
        tangle.update_latest_milestone_index(MilestoneIndex(*ledger_index));

        // TODO should be done in config directly ?
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

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(LedgerWorkerEvent(message_id)) = receiver.next().await {
                if let Err(e) = confirm::<N>(&tangle, &storage, &bus, message_id, &mut ledger_index).await {
                    error!("Confirmation error on {}: {}.", message_id, e);
                    panic!("Aborting due to unexpected ledger error.");
                }

                if !tangle.is_confirmed() {
                    continue;
                }

                if should_snapshot(&tangle, MilestoneIndex(*ledger_index), depth, &snapshot_config) {
                    // if let Err(e) = snapshot(snapshot_config.path(), event.index - depth) {
                    //     error!("Failed to create snapshot: {:?}.", e);
                    // }
                }

                if should_prune(&tangle, MilestoneIndex(*ledger_index), delay, &pruning_config) {
                    // if let Err(e) = prune_database(&tangle, MilestoneIndex(*event.index - delay)) {
                    //     error!("Failed to prune database: {:?}.", e);
                    // }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
