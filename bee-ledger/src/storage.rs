// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    balance::{Balance, BalanceDiffs},
    error::Error,
    model::{OutputDiff, Receipt, TreasuryOutput, Unspent},
};

use bee_message::{
    address::{Address, Ed25519Address},
    ledger_index::LedgerIndex,
    milestone::MilestoneIndex,
    output::{ConsumedOutput, CreatedOutput, Output, OutputId},
};
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert},
    backend,
};

use std::collections::HashMap;

// TODO check all requirements

pub trait StorageBackend:
    backend::StorageBackend
    + BatchBuilder
    + Batch<OutputId, CreatedOutput>
    + Batch<OutputId, ConsumedOutput>
    + Batch<Unspent, ()>
    + Batch<(), LedgerIndex>
    + Batch<MilestoneIndex, OutputDiff>
    + Batch<(Ed25519Address, OutputId), ()>
    + Batch<Address, Balance>
    + Batch<(MilestoneIndex, Receipt), ()>
    + Delete<OutputId, CreatedOutput>
    + Delete<OutputId, ConsumedOutput>
    + Delete<Unspent, ()>
    + Delete<(), LedgerIndex>
    + Exist<OutputId, CreatedOutput>
    + Exist<OutputId, ConsumedOutput>
    + Exist<Unspent, ()>
    + Exist<(), LedgerIndex>
    + Fetch<OutputId, CreatedOutput>
    + Fetch<OutputId, ConsumedOutput>
    + Fetch<(), LedgerIndex>
    + Fetch<Address, Balance>
    + Fetch<bool, Vec<TreasuryOutput>>
    + Insert<OutputId, CreatedOutput>
    + Insert<OutputId, ConsumedOutput>
    + Insert<Unspent, ()>
    + Insert<(), LedgerIndex>
    + for<'a> AsStream<'a, Unspent, ()>
    + for<'a> AsStream<'a, Address, Balance>
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
        + Batch<(Ed25519Address, OutputId), ()>
        + Batch<Address, Balance>
        + Batch<(MilestoneIndex, Receipt), ()>
        + Delete<OutputId, CreatedOutput>
        + Delete<OutputId, ConsumedOutput>
        + Delete<Unspent, ()>
        + Delete<(), LedgerIndex>
        + Exist<OutputId, CreatedOutput>
        + Exist<OutputId, ConsumedOutput>
        + Exist<Unspent, ()>
        + Exist<(), LedgerIndex>
        + Fetch<OutputId, CreatedOutput>
        + Fetch<OutputId, ConsumedOutput>
        + Fetch<(), LedgerIndex>
        + Fetch<Address, Balance>
        + Fetch<bool, Vec<TreasuryOutput>>
        + Insert<OutputId, CreatedOutput>
        + Insert<OutputId, ConsumedOutput>
        + Insert<Unspent, ()>
        + Insert<(), LedgerIndex>
        + for<'a> AsStream<'a, Unspent, ()>
        + for<'a> AsStream<'a, Address, Balance>
        + bee_tangle::storage::StorageBackend
{
}

pub fn create_address_output_relation_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    address: &Address,
    output_id: &OutputId,
) -> Result<(), Error> {
    match address {
        Address::Ed25519(address) => {
            Batch::<(Ed25519Address, OutputId), ()>::batch_insert(storage, batch, &(*address, *output_id), &())
                .map_err(|e| Error::Storage(Box::new(e)))?;
        }
        _ => return Err(Error::UnsupportedAddressType),
    }

    Ok(())
}

pub fn create_output_batch<B: StorageBackend>(
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
            create_address_output_relation_batch(storage, batch, output.address(), output_id)?
        }
        Output::SignatureLockedDustAllowance(output) => {
            create_address_output_relation_batch(storage, batch, output.address(), output_id)?
        }
        _ => return Err(Error::UnsupportedOutputType),
    }

    Ok(())
}

pub async fn create_output<B: StorageBackend>(
    storage: &B,
    output_id: &OutputId,
    output: &CreatedOutput,
) -> Result<(), Error> {
    let mut batch = B::batch_begin();

    create_output_batch(storage, &mut batch, output_id, output)?;

    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub fn consume_output_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    output_id: &OutputId,
    output: &ConsumedOutput,
) -> Result<(), Error> {
    Batch::<OutputId, ConsumedOutput>::batch_insert(storage, batch, output_id, output)
        .map_err(|e| Error::Storage(Box::new(e)))?;
    Batch::<Unspent, ()>::batch_delete(storage, batch, &(*output_id).into())
        .map_err(|e| Error::Storage(Box::new(e)))?;

    Ok(())
}

pub async fn store_balance_diffs<B: StorageBackend>(storage: &B, balance_diffs: &BalanceDiffs) -> Result<(), Error> {
    let mut batch = B::batch_begin();

    store_balance_diffs_batch(storage, &mut batch, balance_diffs).await?;

    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub async fn store_balance_diffs_batch<B: StorageBackend>(
    storage: &B,
    batch: &mut <B as BatchBuilder>::Batch,
    balance_diffs: &BalanceDiffs,
) -> Result<(), Error> {
    for (address, diff) in balance_diffs.iter() {
        let balance = fetch_balance_or_default(storage, address).await? + diff;

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

pub async fn apply_outputs_diff<B: StorageBackend>(
    storage: &B,
    index: MilestoneIndex,
    created_outputs: &HashMap<OutputId, CreatedOutput>,
    consumed_outputs: &HashMap<OutputId, ConsumedOutput>,
    balance_diffs: &BalanceDiffs,
    receipt: &Option<Receipt>,
) -> Result<(), Error> {
    let mut batch = B::batch_begin();

    let mut created_output_ids = Vec::with_capacity(created_outputs.len());
    let mut consumed_output_ids = Vec::with_capacity(consumed_outputs.len());

    Batch::<(), LedgerIndex>::batch_insert(storage, &mut batch, &(), &index.into())
        .map_err(|e| Error::Storage(Box::new(e)))?;

    for (output_id, output) in created_outputs.iter() {
        create_output_batch(storage, &mut batch, output_id, output)?;
        created_output_ids.push(*output_id);
    }

    for (output_id, output) in consumed_outputs.iter() {
        consume_output_batch(storage, &mut batch, output_id, output)?;
        consumed_output_ids.push(*output_id);
    }

    store_balance_diffs_batch(storage, &mut batch, balance_diffs).await?;

    Batch::<MilestoneIndex, OutputDiff>::batch_insert(
        storage,
        &mut batch,
        &index,
        &OutputDiff::new(created_output_ids, consumed_output_ids, None),
    )
    .map_err(|e| Error::Storage(Box::new(e)))?;

    if let Some(receipt) = receipt {
        Batch::<(MilestoneIndex, Receipt), ()>::batch_insert(
            storage,
            &mut batch,
            &(receipt.inner().index().into(), receipt.clone()),
            &(),
        )
        .map_err(|e| Error::Storage(Box::new(e)))?;
    }

    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub async fn rollback_outputs_diff<B: StorageBackend>(
    storage: &B,
    index: MilestoneIndex,
    created_outputs: &HashMap<OutputId, CreatedOutput>,
    consumed_outputs: &HashMap<OutputId, ConsumedOutput>,
) -> Result<(), Error> {
    let mut batch = B::batch_begin();

    Batch::<(), LedgerIndex>::batch_insert(storage, &mut batch, &(), &((index - MilestoneIndex(1)).into()))
        .map_err(|e| Error::Storage(Box::new(e)))?;

    for (output_id, _) in created_outputs.iter() {
        Batch::<OutputId, CreatedOutput>::batch_delete(storage, &mut batch, output_id)
            .map_err(|e| Error::Storage(Box::new(e)))?;
        Batch::<Unspent, ()>::batch_delete(storage, &mut batch, &(*output_id).into())
            .map_err(|e| Error::Storage(Box::new(e)))?;
    }

    for (output_id, _spent) in consumed_outputs.iter() {
        Batch::<OutputId, ConsumedOutput>::batch_delete(storage, &mut batch, output_id)
            .map_err(|e| Error::Storage(Box::new(e)))?;
        Batch::<Unspent, ()>::batch_insert(storage, &mut batch, &(*output_id).into(), &())
            .map_err(|e| Error::Storage(Box::new(e)))?;
    }

    Batch::<MilestoneIndex, OutputDiff>::batch_delete(storage, &mut batch, &index)
        .map_err(|e| Error::Storage(Box::new(e)))?;

    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

#[allow(dead_code)]
pub(crate) async fn fetch_ledger_index<B: StorageBackend>(storage: &B) -> Result<Option<LedgerIndex>, Error> {
    Fetch::<(), LedgerIndex>::fetch(storage, &())
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

#[allow(dead_code)]
pub(crate) async fn insert_ledger_index<B: StorageBackend>(storage: &B, index: &LedgerIndex) -> Result<(), Error> {
    Insert::<(), LedgerIndex>::insert(storage, &(), index)
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

pub(crate) async fn is_output_unspent<B: StorageBackend>(storage: &B, output_id: &OutputId) -> Result<bool, Error> {
    Exist::<Unspent, ()>::exist(storage, &(*output_id).into())
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub async fn fetch_unspent_treasury_output<B: StorageBackend>(storage: &B) -> Result<TreasuryOutput, Error> {
    match Fetch::<bool, Vec<TreasuryOutput>>::fetch(storage, &false)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))?
    {
        Some(outputs) => {
            match outputs.len() {
                0 => panic!("No unspent treasury output found"),
                // Indexing is fine since length is known
                1 => Ok(outputs[0].clone()),
                _ => panic!("More than one unspent treasury output found"),
            }
        }
        None => panic!("No unspent treasury output found"),
    }
}
