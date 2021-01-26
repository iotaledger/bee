// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    balance::{Balance, BalanceDiffs},
    error::Error,
    model::{OutputDiff, Unspent},
};

use bee_message::{
    ledger_index::LedgerIndex,
    milestone::MilestoneIndex,
    payload::transaction::{Address, ConsumedOutput, CreatedOutput, Ed25519Address, Output, OutputId},
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
            Batch::<(Ed25519Address, OutputId), ()>::batch_insert(storage, batch, &(address.clone(), *output_id), &())
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

pub async fn apply_outputs_diff<B: StorageBackend>(
    storage: &B,
    index: MilestoneIndex,
    created_outputs: &HashMap<OutputId, CreatedOutput>,
    consumed_outputs: &HashMap<OutputId, ConsumedOutput>,
    balances: Option<&BalanceDiffs>,
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

    if let Some(balances) = balances {
        for (address, entry) in balances.iter() {
            let (amount, dust_allowance, dust_output) = fetch_balance(storage, address)
                .await?
                .map(|b| (b.amount() as i64, b.dust_allowance() as i64, b.dust_output() as i64))
                .unwrap_or_default();

            Batch::<Address, Balance>::batch_insert(
                storage,
                &mut batch,
                address,
                &Balance::new(
                    (amount + entry.amount()) as u64,
                    (dust_allowance + entry.dust_allowance()) as u64,
                    (dust_output + entry.dust_output()) as u64,
                ),
            )
            .map_err(|e| Error::Storage(Box::new(e)))?;
        }
    }

    Batch::<MilestoneIndex, OutputDiff>::batch_insert(
        storage,
        &mut batch,
        &index,
        &OutputDiff::new(created_output_ids, consumed_output_ids),
    )
    .map_err(|e| Error::Storage(Box::new(e)))?;

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
