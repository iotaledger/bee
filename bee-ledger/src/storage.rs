// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::Error,
    metadata::WhiteFlagMetadata,
    model::{LedgerIndex, Output, Spent, Unspent},
};

use bee_message::payload::transaction::OutputId;
use bee_protocol::MilestoneIndex;
use bee_storage::{
    access::{Batch, BatchBuilder, Delete, Exist, Fetch, Insert},
    storage,
};

// TODO check all requirements

pub trait Backend:
    storage::Backend
    + BatchBuilder
    + Batch<OutputId, Output>
    + Batch<OutputId, Spent>
    + Batch<Unspent, ()>
    + Batch<(), LedgerIndex>
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
{
}

impl<T> Backend for T where
    T: storage::Backend
        + BatchBuilder
        + Batch<OutputId, Output>
        + Batch<OutputId, Spent>
        + Batch<Unspent, ()>
        + Batch<(), LedgerIndex>
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
{
}

pub(crate) async fn apply_diff<B: Backend>(storage: &B, metadata: &WhiteFlagMetadata) -> Result<(), Error> {
    let mut batch = B::batch_begin();

    Batch::<(), LedgerIndex>::batch_insert(storage, &mut batch, &(), &metadata.index.into())
        .map_err(|e| Error::Storage(Box::new(e)))?;

    for (output_id, spent) in metadata.spent_outputs.iter() {
        Batch::<OutputId, Spent>::batch_insert(storage, &mut batch, output_id, spent)
            .map_err(|e| Error::Storage(Box::new(e)))?;
        Batch::<Unspent, ()>::batch_delete(storage, &mut batch, &(*output_id).into())
            .map_err(|e| Error::Storage(Box::new(e)))?;
    }

    for (output_id, output) in metadata.created_outputs.iter() {
        Batch::<OutputId, Output>::batch_insert(storage, &mut batch, output_id, output)
            .map_err(|e| Error::Storage(Box::new(e)))?;
        Batch::<Unspent, ()>::batch_insert(storage, &mut batch, &(*output_id).into(), &())
            .map_err(|e| Error::Storage(Box::new(e)))?;
    }

    // TODO store diff

    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

#[allow(dead_code)]
pub(crate) async fn rollback_diff<B: Backend>(storage: &B, metadata: &WhiteFlagMetadata) -> Result<(), Error> {
    let mut batch = B::batch_begin();

    Batch::<(), LedgerIndex>::batch_insert(storage, &mut batch, &(), &(MilestoneIndex(*metadata.index - 1)).into())
        .map_err(|e| Error::Storage(Box::new(e)))?;

    for (output_id, _spent) in metadata.spent_outputs.iter() {
        Batch::<OutputId, Spent>::batch_delete(storage, &mut batch, output_id)
            .map_err(|e| Error::Storage(Box::new(e)))?;
        Batch::<Unspent, ()>::batch_insert(storage, &mut batch, &(*output_id).into(), &())
            .map_err(|e| Error::Storage(Box::new(e)))?;
    }

    for (output_id, _) in metadata.created_outputs.iter() {
        Batch::<OutputId, Output>::batch_delete(storage, &mut batch, output_id)
            .map_err(|e| Error::Storage(Box::new(e)))?;
        Batch::<Unspent, ()>::batch_delete(storage, &mut batch, &(*output_id).into())
            .map_err(|e| Error::Storage(Box::new(e)))?;
    }

    // TODO delete diff

    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

#[allow(dead_code)]
pub(crate) async fn fetch_ledger_index<B: Backend>(storage: &B) -> Result<Option<LedgerIndex>, Error> {
    Fetch::<(), LedgerIndex>::fetch(storage, &())
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

#[allow(dead_code)]
pub(crate) async fn insert_ledger_index<B: Backend>(storage: &B, index: &LedgerIndex) -> Result<(), Error> {
    Insert::<(), LedgerIndex>::insert(storage, &(), index)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn fetch_output<B: Backend>(storage: &B, output_id: &OutputId) -> Result<Option<Output>, Error> {
    Fetch::<OutputId, Output>::fetch(storage, output_id)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn is_output_unspent<B: Backend>(storage: &B, output_id: &OutputId) -> Result<bool, Error> {
    Exist::<Unspent, ()>::exist(storage, &(*output_id).into())
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}
