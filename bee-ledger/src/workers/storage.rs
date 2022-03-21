// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing ledger storage operations.

use crate::{
    types::{
        snapshot::SnapshotInfo, ConsumedOutput, CreatedOutput, LedgerIndex, Migration, OutputDiff, Receipt,
        TreasuryDiff, TreasuryOutput, Unspent,
    },
    workers::error::Error,
};

use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    Message, MessageId,
};
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

use std::collections::HashMap;

/// A blanket-implemented helper trait for the storage layer.
pub trait StorageBackend:
    backend::StorageBackend
    + BatchBuilder
    + Batch<OutputId, CreatedOutput>
    + Batch<OutputId, ConsumedOutput>
    + Batch<Unspent, ()>
    + Batch<(), LedgerIndex>
    + Batch<MilestoneIndex, OutputDiff>
    + Batch<(MilestoneIndex, Receipt), ()>
    + Batch<(bool, TreasuryOutput), ()>
    + Batch<SolidEntryPoint, MilestoneIndex>
    + Batch<(MilestoneIndex, UnreferencedMessage), ()>
    + Batch<(MessageId, MessageId), ()>
    + Batch<MessageId, Message>
    + Batch<MessageId, MessageMetadata>
    + Batch<MilestoneIndex, Milestone>
    + Exist<Unspent, ()>
    + Fetch<(), SnapshotInfo>
    + Fetch<OutputId, CreatedOutput>
    + Fetch<(), LedgerIndex>
    + Fetch<bool, Vec<TreasuryOutput>>
    + Fetch<MilestoneIndex, Milestone>
    + Fetch<MilestoneIndex, Vec<Receipt>>
    + Fetch<MilestoneIndex, Vec<UnreferencedMessage>>
    + Fetch<MilestoneIndex, OutputDiff>
    + Insert<(), SnapshotInfo>
    + Insert<(), LedgerIndex>
    + Insert<(bool, TreasuryOutput), ()>
    + Truncate<SolidEntryPoint, MilestoneIndex>
    + for<'a> AsIterator<'a, Unspent, ()>
    + for<'a> AsIterator<'a, SolidEntryPoint, MilestoneIndex>
    + bee_tangle::storage::StorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + BatchBuilder
        + Batch<OutputId, CreatedOutput>
        + Batch<OutputId, ConsumedOutput>
        + Batch<Unspent, ()>
        + Batch<(), LedgerIndex>
        + Batch<MilestoneIndex, OutputDiff>
        + Batch<(MilestoneIndex, Receipt), ()>
        + Batch<(bool, TreasuryOutput), ()>
        + Batch<SolidEntryPoint, MilestoneIndex>
        + Batch<(MilestoneIndex, UnreferencedMessage), ()>
        + Batch<(MessageId, MessageId), ()>
        + Batch<MessageId, Message>
        + Batch<MessageId, MessageMetadata>
        + Batch<MilestoneIndex, Milestone>
        + Exist<Unspent, ()>
        + Fetch<(), SnapshotInfo>
        + Fetch<OutputId, CreatedOutput>
        + Fetch<(), LedgerIndex>
        + Fetch<bool, Vec<TreasuryOutput>>
        + Fetch<MilestoneIndex, Milestone>
        + Fetch<MilestoneIndex, Vec<Receipt>>
        + Fetch<MilestoneIndex, Vec<UnreferencedMessage>>
        + Fetch<MilestoneIndex, OutputDiff>
        + Insert<(), SnapshotInfo>
        + Insert<(), LedgerIndex>
        + Insert<(bool, TreasuryOutput), ()>
        + Truncate<SolidEntryPoint, MilestoneIndex>
        + for<'a> AsIterator<'a, Unspent, ()>
        + for<'a> AsIterator<'a, SolidEntryPoint, MilestoneIndex>
        + bee_tangle::storage::StorageBackend
{
}

pub(crate) fn insert_created_output_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    output_id: &OutputId,
    output: &CreatedOutput,
) -> Result<(), Error> {
    Batch::<OutputId, CreatedOutput>::batch_insert(storage, batch, output_id, output)
        .map_err(|e| Error::Storage(Box::new(e)))?;
    Batch::<Unspent, ()>::batch_insert(storage, batch, &(*output_id).into(), &())
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn delete_created_output_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    output_id: &OutputId,
    _output: &CreatedOutput,
) -> Result<(), Error> {
    Batch::<OutputId, CreatedOutput>::batch_delete(storage, batch, output_id)
        .map_err(|e| Error::Storage(Box::new(e)))?;
    Batch::<Unspent, ()>::batch_delete(storage, batch, &(*output_id).into()).map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn create_output<B: StorageBackend>(
    storage: &B,
    output_id: &OutputId,
    output: &CreatedOutput,
) -> Result<(), Error> {
    let mut batch = B::batch_begin();

    insert_created_output_batch(storage, &mut batch, output_id, output)?;

    storage
        .batch_commit(batch, true)
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn insert_consumed_output_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    output_id: &OutputId,
    output: &ConsumedOutput,
) -> Result<(), Error> {
    Batch::<OutputId, ConsumedOutput>::batch_insert(storage, batch, output_id, output)
        .map_err(|e| Error::Storage(Box::new(e)))?;
    Batch::<Unspent, ()>::batch_delete(storage, batch, &(*output_id).into()).map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn delete_consumed_output_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    output_id: &OutputId,
) -> Result<(), Error> {
    Batch::<OutputId, ConsumedOutput>::batch_delete(storage, batch, output_id)
        .map_err(|e| Error::Storage(Box::new(e)))?;
    Batch::<Unspent, ()>::batch_insert(storage, batch, &(*output_id).into(), &())
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn apply_milestone<B: StorageBackend>(
    storage: &B,
    index: MilestoneIndex,
    created_outputs: &HashMap<OutputId, CreatedOutput>,
    consumed_outputs: &HashMap<OutputId, (CreatedOutput, ConsumedOutput)>,
    migration: &Option<Migration>,
) -> Result<(), Error> {
    let mut batch = B::batch_begin();

    insert_ledger_index_batch(storage, &mut batch, &index.into())?;

    let created_output_ids = created_outputs
        .iter()
        .map::<Result<_, Error>, _>(|(output_id, output)| {
            insert_created_output_batch(storage, &mut batch, output_id, output)?;
            Ok(*output_id)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let consumed_output_ids = consumed_outputs
        .iter()
        .map::<Result<_, Error>, _>(|(output_id, (_, consumed_output))| {
            insert_consumed_output_batch(storage, &mut batch, output_id, consumed_output)?;
            Ok(*output_id)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let treasury_diff = if let Some(migration) = migration {
        insert_receipt_batch(storage, &mut batch, migration.receipt())?;
        insert_treasury_output_batch(storage, &mut batch, migration.created_treasury())?;
        spend_treasury_output_batch(storage, &mut batch, migration.consumed_treasury())?;

        Some(TreasuryDiff::new(
            *migration.created_treasury().milestone_id(),
            *migration.consumed_treasury().milestone_id(),
        ))
    } else {
        None
    };

    Batch::<MilestoneIndex, OutputDiff>::batch_insert(
        storage,
        &mut batch,
        &index,
        &OutputDiff::new(created_output_ids, consumed_output_ids, treasury_diff)
            .map_err(|e| Error::Storage(Box::new(e)))?,
    )
    .map_err(|e| Error::Storage(Box::new(e)))?;

    storage
        .batch_commit(batch, true)
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn rollback_milestone<B: StorageBackend>(
    storage: &B,
    index: MilestoneIndex,
    created_outputs: &HashMap<OutputId, CreatedOutput>,
    consumed_outputs: &HashMap<OutputId, (CreatedOutput, ConsumedOutput)>,
    migration: &Option<Migration>,
) -> Result<(), Error> {
    let mut batch = B::batch_begin();

    insert_ledger_index_batch(storage, &mut batch, &((index - 1).into()))?;

    for (output_id, created_output) in created_outputs.iter() {
        delete_created_output_batch(storage, &mut batch, output_id, created_output)?;
    }

    for (output_id, (created_output, _)) in consumed_outputs.iter() {
        insert_created_output_batch(storage, &mut batch, output_id, created_output)?;
        delete_consumed_output_batch(storage, &mut batch, output_id)?;
    }

    if let Some(migration) = migration {
        delete_receipt_batch(storage, &mut batch, migration.receipt())?;
        delete_treasury_output_batch(storage, &mut batch, migration.created_treasury())?;
        unspend_treasury_output_batch(storage, &mut batch, migration.consumed_treasury())?;
    }

    Batch::<MilestoneIndex, OutputDiff>::batch_delete(storage, &mut batch, &index)
        .map_err(|e| Error::Storage(Box::new(e)))?;

    storage
        .batch_commit(batch, true)
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn insert_ledger_index<B: StorageBackend>(storage: &B, index: &LedgerIndex) -> Result<(), Error> {
    Insert::<(), LedgerIndex>::insert(storage, &(), index).map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn insert_ledger_index_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    index: &LedgerIndex,
) -> Result<(), Error> {
    Batch::<(), LedgerIndex>::batch_insert(storage, batch, &(), index).map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn fetch_ledger_index<B: StorageBackend>(storage: &B) -> Result<Option<LedgerIndex>, Error> {
    Fetch::<(), LedgerIndex>::fetch(storage, &()).map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn insert_receipt_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    receipt: &Receipt,
) -> Result<(), Error> {
    Batch::<(MilestoneIndex, Receipt), ()>::batch_insert(
        storage,
        batch,
        &(receipt.inner().migrated_at(), receipt.clone()),
        &(),
    )
    .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn delete_receipt_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    receipt: &Receipt,
) -> Result<(), Error> {
    Batch::<(MilestoneIndex, Receipt), ()>::batch_delete(
        storage,
        batch,
        &(receipt.inner().migrated_at(), receipt.clone()),
    )
    .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn insert_snapshot_info<B: StorageBackend>(storage: &B, snapshot_info: &SnapshotInfo) -> Result<(), Error> {
    Insert::<(), SnapshotInfo>::insert(&*storage, &(), snapshot_info).map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn fetch_snapshot_info<B: StorageBackend>(storage: &B) -> Result<Option<SnapshotInfo>, Error> {
    Fetch::<(), SnapshotInfo>::fetch(storage, &()).map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn fetch_output<B: StorageBackend>(
    storage: &B,
    output_id: &OutputId,
) -> Result<Option<CreatedOutput>, Error> {
    Fetch::<OutputId, CreatedOutput>::fetch(storage, output_id).map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn is_output_unspent<B: StorageBackend>(storage: &B, output_id: &OutputId) -> Result<bool, Error> {
    Exist::<Unspent, ()>::exist(storage, &(*output_id).into()).map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn insert_treasury_output<B: StorageBackend>(
    storage: &B,
    treasury_output: &TreasuryOutput,
) -> Result<(), Error> {
    Insert::<(bool, TreasuryOutput), ()>::insert(storage, &(false, treasury_output.clone()), &())
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn insert_treasury_output_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    treasury_output: &TreasuryOutput,
) -> Result<(), Error> {
    Batch::<(bool, TreasuryOutput), ()>::batch_insert(storage, batch, &(false, treasury_output.clone()), &())
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn delete_treasury_output_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    treasury_output: &TreasuryOutput,
) -> Result<(), Error> {
    Batch::<(bool, TreasuryOutput), ()>::batch_delete(storage, batch, &(false, treasury_output.clone()))
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn spend_treasury_output_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    treasury_output: &TreasuryOutput,
) -> Result<(), Error> {
    Batch::<(bool, TreasuryOutput), ()>::batch_insert(storage, batch, &(true, treasury_output.clone()), &())
        .map_err(|e| Error::Storage(Box::new(e)))?;
    Batch::<(bool, TreasuryOutput), ()>::batch_delete(storage, batch, &(false, treasury_output.clone()))
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn unspend_treasury_output_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    treasury_output: &TreasuryOutput,
) -> Result<(), Error> {
    Batch::<(bool, TreasuryOutput), ()>::batch_insert(storage, batch, &(false, treasury_output.clone()), &())
        .map_err(|e| Error::Storage(Box::new(e)))?;
    Batch::<(bool, TreasuryOutput), ()>::batch_delete(storage, batch, &(true, treasury_output.clone()))
        .map_err(|e| Error::Storage(Box::new(e)))
}

/// Fetches the unspent treasury output from the storage.
pub fn fetch_unspent_treasury_output<B: StorageBackend>(storage: &B) -> Result<TreasuryOutput, Error> {
    if let Some(outputs) =
        Fetch::<bool, Vec<TreasuryOutput>>::fetch(storage, &false).map_err(|e| Error::Storage(Box::new(e)))?
    {
        match outputs.as_slice() {
            // There has to be an unspent treasury output at all time.
            [] => panic!("No unspent treasury output found"),
            [output] => Ok(output.clone()),
            // There should be one and only one unspent treasury output at all time.
            _ => panic!("More than one unspent treasury output found"),
        }
    } else {
        // There has to be an unspent treasury output at all time.
        panic!("No unspent treasury output found");
    }
}
