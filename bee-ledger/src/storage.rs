// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::Error,
    model::{Output, OutputDiff, Spent, Unspent},
    IOTA_SUPPLY,
};

use bee_message::{ledger_index::LedgerIndex, milestone::MilestoneIndex, payload::transaction::OutputId};
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert},
    backend,
};

use futures::StreamExt;

use std::collections::HashMap;

// TODO check all requirements

pub trait StorageBackend:
    backend::StorageBackend
    + BatchBuilder
    + Batch<OutputId, Output>
    + Batch<OutputId, Spent>
    + Batch<Unspent, ()>
    + Batch<(), LedgerIndex>
    + Batch<MilestoneIndex, OutputDiff>
    + Delete<OutputId, Output>
    + Delete<OutputId, Spent>
    + Delete<Unspent, ()>
    + Delete<(), LedgerIndex>
    + Exist<OutputId, Output>
    + Exist<OutputId, Spent>
    + Exist<Unspent, ()>
    + Exist<(), LedgerIndex>
    + Fetch<OutputId, Output>
    + Fetch<OutputId, Spent>
    + Fetch<(), LedgerIndex>
    + Insert<OutputId, Output>
    + Insert<OutputId, Spent>
    + Insert<Unspent, ()>
    + Insert<(), LedgerIndex>
    + for<'a> AsStream<'a, Unspent, ()>
    + bee_tangle::storage::StorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + BatchBuilder
        + Batch<OutputId, Output>
        + Batch<OutputId, Spent>
        + Batch<Unspent, ()>
        + Batch<(), LedgerIndex>
        + Batch<MilestoneIndex, OutputDiff>
        + Delete<OutputId, Output>
        + Delete<OutputId, Spent>
        + Delete<Unspent, ()>
        + Delete<(), LedgerIndex>
        + Exist<OutputId, Output>
        + Exist<OutputId, Spent>
        + Exist<Unspent, ()>
        + Exist<(), LedgerIndex>
        + Fetch<OutputId, Output>
        + Fetch<OutputId, Spent>
        + Fetch<(), LedgerIndex>
        + Insert<OutputId, Output>
        + Insert<OutputId, Spent>
        + Insert<Unspent, ()>
        + Insert<(), LedgerIndex>
        + for<'a> AsStream<'a, Unspent, ()>
        + bee_tangle::storage::StorageBackend
{
}

pub async fn apply_diff<B: StorageBackend>(
    storage: &B,
    index: MilestoneIndex,
    spent_outputs: &HashMap<OutputId, Spent>,
    created_outputs: &HashMap<OutputId, Output>,
) -> Result<(), Error> {
    let mut batch = B::batch_begin();

    let mut spent_output_ids = Vec::with_capacity(spent_outputs.len());
    let mut created_outputs_ids = Vec::with_capacity(created_outputs.len());

    Batch::<(), LedgerIndex>::batch_insert(storage, &mut batch, &(), &index.into())
        .map_err(|e| Error::Storage(Box::new(e)))?;

    for (output_id, spent) in spent_outputs.iter() {
        Batch::<OutputId, Spent>::batch_insert(storage, &mut batch, output_id, spent)
            .map_err(|e| Error::Storage(Box::new(e)))?;
        Batch::<Unspent, ()>::batch_delete(storage, &mut batch, &(*output_id).into())
            .map_err(|e| Error::Storage(Box::new(e)))?;
        spent_output_ids.push(*output_id);
    }

    for (output_id, output) in created_outputs.iter() {
        Batch::<OutputId, Output>::batch_insert(storage, &mut batch, output_id, output)
            .map_err(|e| Error::Storage(Box::new(e)))?;
        Batch::<Unspent, ()>::batch_insert(storage, &mut batch, &(*output_id).into(), &())
            .map_err(|e| Error::Storage(Box::new(e)))?;
        created_outputs_ids.push(*output_id);
        // TODO address mapping
    }

    Batch::<MilestoneIndex, OutputDiff>::batch_insert(
        storage,
        &mut batch,
        &index,
        &OutputDiff::new(spent_output_ids, created_outputs_ids),
    )
    .map_err(|e| Error::Storage(Box::new(e)))?;

    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub async fn rollback_diff<B: StorageBackend>(
    storage: &B,
    index: MilestoneIndex,
    spent_outputs: &HashMap<OutputId, Spent>,
    created_outputs: &HashMap<OutputId, Output>,
) -> Result<(), Error> {
    let mut batch = B::batch_begin();

    Batch::<(), LedgerIndex>::batch_insert(storage, &mut batch, &(), &((index - MilestoneIndex(1)).into()))
        .map_err(|e| Error::Storage(Box::new(e)))?;

    for (output_id, _spent) in spent_outputs.iter() {
        Batch::<OutputId, Spent>::batch_delete(storage, &mut batch, output_id)
            .map_err(|e| Error::Storage(Box::new(e)))?;
        Batch::<Unspent, ()>::batch_insert(storage, &mut batch, &(*output_id).into(), &())
            .map_err(|e| Error::Storage(Box::new(e)))?;
    }

    for (output_id, _) in created_outputs.iter() {
        Batch::<OutputId, Output>::batch_delete(storage, &mut batch, output_id)
            .map_err(|e| Error::Storage(Box::new(e)))?;
        Batch::<Unspent, ()>::batch_delete(storage, &mut batch, &(*output_id).into())
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

#[allow(dead_code)]
pub(crate) async fn insert_ledger_index<B: StorageBackend>(storage: &B, index: &LedgerIndex) -> Result<(), Error> {
    Insert::<(), LedgerIndex>::insert(storage, &(), index)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn fetch_output<B: StorageBackend>(
    storage: &B,
    output_id: &OutputId,
) -> Result<Option<Output>, Error> {
    Fetch::<OutputId, Output>::fetch(storage, output_id)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn is_output_unspent<B: StorageBackend>(storage: &B, output_id: &OutputId) -> Result<bool, Error> {
    Exist::<Unspent, ()>::exist(storage, &(*output_id).into())
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub async fn check_ledger_state<B: StorageBackend>(storage: &B) -> Result<bool, Error> {
    let mut total: u64 = 0;
    let mut stream = AsStream::<Unspent, ()>::stream(storage)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))?;

    while let Some((unspent, _)) = stream.next().await {
        // Unwrap: an unspent output has to be in database.
        let output = fetch_output(storage, &*unspent).await?.unwrap();

        match output.inner() {
            bee_message::payload::transaction::Output::SignatureLockedSingle(output) => {
                total += output.amount();
            }
            // TODO
            _ => panic!("unsupported output"),
        }
    }

    Ok(total == IOTA_SUPPLY)
}
