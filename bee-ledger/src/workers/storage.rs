// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing ledger storage operations.

use crate::{
    types::{
        snapshot::SnapshotInfo, Balance, BalanceDiffs, ConsumedOutput, CreatedOutput, LedgerIndex, Migration,
        OutputDiff, Receipt, TreasuryDiff, TreasuryOutput, Unspent,
    },
    workers::error::Error,
};

use bee_message::{
    address::{Address, Ed25519Address},
    milestone::{Milestone, MilestoneIndex},
    output::{Output, OutputId},
    prelude::PaddedIndex,
    Message, MessageId,
};
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Exist, Fetch, Insert, Truncate},
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
    // === Batch operations ===
    + Batch<(), LedgerIndex>
    + Batch<(bool, TreasuryOutput), ()>
    + Batch<(Ed25519Address, OutputId), ()>
    + Batch<(MessageId, MessageId), ()>
    + Batch<(MilestoneIndex, Receipt), ()>
    + Batch<(MilestoneIndex, UnreferencedMessage), ()>
    + Batch<(PaddedIndex, MessageId), ()>
    + Batch<Address, Balance>
    + Batch<MessageId, Message>
    + Batch<MessageId, MessageMetadata>
    + Batch<MilestoneIndex, Milestone>
    + Batch<MilestoneIndex, OutputDiff>
    + Batch<OutputId, CreatedOutput>
    + Batch<OutputId, ConsumedOutput>
    + Batch<SolidEntryPoint, MilestoneIndex>
    + Batch<Unspent, ()>
    // === Exist operations ===
    + Exist<Unspent, ()>
    // === Fetch operations ===
    + Fetch<(), LedgerIndex>
    + Fetch<(), SnapshotInfo>
    + Fetch<Address, Balance>
    + Fetch<bool, Vec<TreasuryOutput>>
    + Fetch<Ed25519Address, Vec<OutputId>>
    + Fetch<MessageId, Message>
    + Fetch<MessageId, MessageMetadata>
    + Fetch<MessageId, Vec<MessageId>>
    + Fetch<MilestoneIndex, Milestone>
    + Fetch<MilestoneIndex, Vec<Receipt>>
    + Fetch<MilestoneIndex, Vec<UnreferencedMessage>>
    + Fetch<OutputId, CreatedOutput>
    + Fetch<SolidEntryPoint, MilestoneIndex>
    // === Insert operations ===
    + Insert<(), SnapshotInfo>
    + Insert<(), LedgerIndex>
    + Insert<(bool, TreasuryOutput), ()>
    + Insert<SolidEntryPoint, MilestoneIndex>
    // === Truncate operations ===
    + Truncate<SolidEntryPoint, MilestoneIndex>
    // === Stream operations ===
    + for<'a> AsStream<'a, Unspent, ()>
    + for<'a> AsStream<'a, Address, Balance>
    + for<'a> AsStream<'a, SolidEntryPoint, MilestoneIndex>
    // === Tangle operations ===
    + bee_tangle::storage::StorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + BatchBuilder
        // === Batch operations ===
        + Batch<(), LedgerIndex>
        + Batch<(Ed25519Address, OutputId), ()>
        + Batch<(bool, TreasuryOutput), ()>
        + Batch<(MessageId, MessageId), ()>
        + Batch<(MilestoneIndex, Receipt), ()>
        + Batch<(MilestoneIndex, UnreferencedMessage), ()>
        + Batch<(PaddedIndex, MessageId), ()>
        + Batch<Address, Balance>
        + Batch<MessageId, Message>
        + Batch<MessageId, MessageMetadata>
        + Batch<MilestoneIndex, Milestone>
        + Batch<MilestoneIndex, OutputDiff>
        + Batch<OutputId, CreatedOutput>
        + Batch<OutputId, ConsumedOutput>
        + Batch<SolidEntryPoint, MilestoneIndex>
        + Batch<Unspent, ()>
        // === Exist operations ===
        + Exist<Unspent, ()>
        // === Fetch operations ===
        + Fetch<(), LedgerIndex>
        + Fetch<(), SnapshotInfo>
        + Fetch<Address, Balance>
        + Fetch<bool, Vec<TreasuryOutput>>
        + Fetch<Ed25519Address, Vec<OutputId>>
        + Fetch<MessageId, Message>
        + Fetch<MessageId, MessageMetadata>
        + Fetch<MessageId, Vec<MessageId>>
        + Fetch<MilestoneIndex, Milestone>
        + Fetch<MilestoneIndex, Vec<Receipt>>
        + Fetch<MilestoneIndex, Vec<UnreferencedMessage>>
        + Fetch<OutputId, CreatedOutput>
        + Fetch<SolidEntryPoint, MilestoneIndex>
        // === Insert operations ===
        + Insert<(), SnapshotInfo>
        + Insert<(), LedgerIndex>
        + Insert<(bool, TreasuryOutput), ()>
        + Insert<SolidEntryPoint, MilestoneIndex>
        // === Truncate operations ===
        + Truncate<SolidEntryPoint, MilestoneIndex>
        // === Stream operations ===
        + for<'a> AsStream<'a, Unspent, ()>
        + for<'a> AsStream<'a, Address, Balance>
        + for<'a> AsStream<'a, SolidEntryPoint, MilestoneIndex>
        // === Tangle operations ===
        + bee_tangle::storage::StorageBackend
{
}

pub(crate) fn insert_output_id_for_address_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    address: &Address,
    output_id: &OutputId,
) -> Result<(), Error> {
    match address {
        Address::Ed25519(address) => {
            Batch::<(Ed25519Address, OutputId), ()>::batch_insert(storage, batch, &(*address, *output_id), &())
                .map_err(|e| Error::Storage(Box::new(e)))
        }
    }
}

pub(crate) fn delete_output_id_for_address_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    address: &Address,
    output_id: &OutputId,
) -> Result<(), Error> {
    match address {
        Address::Ed25519(address) => {
            Batch::<(Ed25519Address, OutputId), ()>::batch_delete(storage, batch, &(*address, *output_id))
                .map_err(|e| Error::Storage(Box::new(e)))
        }
    }
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
        .map_err(|e| Error::Storage(Box::new(e)))?;

    match output.inner() {
        Output::SignatureLockedSingle(output) => {
            insert_output_id_for_address_batch(storage, batch, output.address(), output_id)
        }
        Output::SignatureLockedDustAllowance(output) => {
            insert_output_id_for_address_batch(storage, batch, output.address(), output_id)
        }
        Output::Treasury(_) => Err(Error::UnsupportedOutputKind(output.kind())),
    }
}

pub(crate) fn delete_created_output_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    output_id: &OutputId,
    output: &CreatedOutput,
) -> Result<(), Error> {
    Batch::<OutputId, CreatedOutput>::batch_delete(storage, batch, output_id)
        .map_err(|e| Error::Storage(Box::new(e)))?;
    Batch::<Unspent, ()>::batch_delete(storage, batch, &(*output_id).into())
        .map_err(|e| Error::Storage(Box::new(e)))?;

    match output.inner() {
        Output::SignatureLockedSingle(output) => {
            delete_output_id_for_address_batch(storage, batch, output.address(), output_id)
        }
        Output::SignatureLockedDustAllowance(output) => {
            delete_output_id_for_address_batch(storage, batch, output.address(), output_id)
        }
        Output::Treasury(_) => Err(Error::UnsupportedOutputKind(output.kind())),
    }
}

pub(crate) async fn create_output<B: StorageBackend>(
    storage: &B,
    output_id: &OutputId,
    output: &CreatedOutput,
) -> Result<(), Error> {
    let mut batch = B::batch_begin();

    insert_created_output_batch(storage, &mut batch, output_id, output)?;

    storage
        .batch_commit(batch, true)
        .await
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

pub(crate) async fn apply_balance_diffs<B: StorageBackend>(
    storage: &B,
    balance_diffs: &BalanceDiffs,
) -> Result<(), Error> {
    let mut batch = B::batch_begin();

    apply_balance_diffs_batch(storage, &mut batch, balance_diffs).await?;

    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn apply_balance_diffs_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    balance_diffs: &BalanceDiffs,
) -> Result<(), Error> {
    for (address, diff) in balance_diffs.iter() {
        let balance = fetch_balance_or_default(storage, address).await?.apply_diff(diff)?;

        if balance.amount() != 0 {
            Batch::<Address, Balance>::batch_insert(storage, batch, address, &balance)
                .map_err(|e| Error::Storage(Box::new(e)))?;
        } else {
            Batch::<Address, Balance>::batch_delete(storage, batch, address)
                .map_err(|e| Error::Storage(Box::new(e)))?;
        }
    }

    Ok(())
}

pub(crate) async fn apply_milestone<B: StorageBackend>(
    storage: &B,
    index: MilestoneIndex,
    created_outputs: &HashMap<OutputId, CreatedOutput>,
    consumed_outputs: &HashMap<OutputId, (CreatedOutput, ConsumedOutput)>,
    balance_diffs: &BalanceDiffs,
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

    apply_balance_diffs_batch(storage, &mut batch, balance_diffs).await?;

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
        &OutputDiff::new(created_output_ids, consumed_output_ids, treasury_diff),
    )
    .map_err(|e| Error::Storage(Box::new(e)))?;

    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn rollback_milestone<B: StorageBackend>(
    storage: &B,
    index: MilestoneIndex,
    created_outputs: &HashMap<OutputId, CreatedOutput>,
    consumed_outputs: &HashMap<OutputId, (CreatedOutput, ConsumedOutput)>,
    balance_diffs: &BalanceDiffs,
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

    apply_balance_diffs_batch(storage, &mut batch, &balance_diffs.negated()).await?;

    if let Some(migration) = migration {
        delete_receipt_batch(storage, &mut batch, migration.receipt())?;
        delete_treasury_output_batch(storage, &mut batch, migration.created_treasury())?;
        unspend_treasury_output_batch(storage, &mut batch, migration.consumed_treasury())?;
    }

    Batch::<MilestoneIndex, OutputDiff>::batch_delete(storage, &mut batch, &index)
        .map_err(|e| Error::Storage(Box::new(e)))?;

    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn fetch_balance<B: StorageBackend>(storage: &B, address: &Address) -> Result<Option<Balance>, Error> {
    Fetch::<Address, Balance>::fetch(storage, address)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn fetch_balance_or_default<B: StorageBackend>(
    storage: &B,
    address: &Address,
) -> Result<Balance, Error> {
    Ok(fetch_balance(storage, address).await?.unwrap_or_default())
}

pub(crate) async fn insert_ledger_index<B: StorageBackend>(storage: &B, index: &LedgerIndex) -> Result<(), Error> {
    Insert::<(), LedgerIndex>::insert(storage, &(), index)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) fn insert_ledger_index_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    index: &LedgerIndex,
) -> Result<(), Error> {
    Batch::<(), LedgerIndex>::batch_insert(storage, batch, &(), index).map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn fetch_ledger_index<B: StorageBackend>(storage: &B) -> Result<Option<LedgerIndex>, Error> {
    Fetch::<(), LedgerIndex>::fetch(storage, &())
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
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

pub(crate) async fn insert_snapshot_info<B: StorageBackend>(
    storage: &B,
    snapshot_info: &SnapshotInfo,
) -> Result<(), Error> {
    Insert::<(), SnapshotInfo>::insert(&*storage, &(), snapshot_info)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn fetch_snapshot_info<B: StorageBackend>(storage: &B) -> Result<Option<SnapshotInfo>, Error> {
    Fetch::<(), SnapshotInfo>::fetch(storage, &())
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn fetch_output<B: StorageBackend>(
    storage: &B,
    output_id: &OutputId,
) -> Result<Option<CreatedOutput>, Error> {
    Fetch::<OutputId, CreatedOutput>::fetch(storage, output_id)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn fetch_outputs_for_ed25519_address<B: StorageBackend>(
    storage: &B,
    address: &Ed25519Address,
) -> Result<Option<Vec<OutputId>>, Error> {
    Fetch::<Ed25519Address, Vec<OutputId>>::fetch(&*storage, &address)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn is_output_unspent<B: StorageBackend>(storage: &B, output_id: &OutputId) -> Result<bool, Error> {
    Exist::<Unspent, ()>::exist(storage, &(*output_id).into())
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn insert_treasury_output<B: StorageBackend>(
    storage: &B,
    treasury_output: &TreasuryOutput,
) -> Result<(), Error> {
    Insert::<(bool, TreasuryOutput), ()>::insert(storage, &(false, treasury_output.clone()), &())
        .await
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
pub async fn fetch_unspent_treasury_output<B: StorageBackend>(storage: &B) -> Result<TreasuryOutput, Error> {
    if let Some(outputs) = Fetch::<bool, Vec<TreasuryOutput>>::fetch(storage, &false)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))?
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
