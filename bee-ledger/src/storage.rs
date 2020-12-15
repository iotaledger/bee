// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::Error,
    metadata::WhiteFlagMetadata,
    model::{LedgerIndex, Output, Spent, Unspent},
};

use bee_message::payload::transaction::OutputId;
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

pub(crate) async fn apply_metadata<B: Backend>(storage: &B, metadata: &WhiteFlagMetadata) -> Result<(), Error> {
    let mut batch = B::batch_begin();

    storage
        .batch_insert(&mut batch, &(), &LedgerIndex::new(metadata.index))
        .map_err(|e| Error::Storage(Box::new(e)))?;

    for (output_id, spent) in metadata.spent_outputs.iter() {
        storage
            .batch_insert(&mut batch, output_id, spent)
            .map_err(|e| Error::Storage(Box::new(e)))?;
    }

    for (output_id, output) in metadata.created_outputs.iter() {
        storage
            .batch_insert(&mut batch, output_id, output)
            .map_err(|e| Error::Storage(Box::new(e)))?;
    }

    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn get_output<B: Backend>(storage: &B, output_id: &OutputId) -> Result<Option<Output>, Error> {
    Fetch::<OutputId, Output>::fetch(storage, output_id)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

pub(crate) async fn is_output_unspent<B: Backend>(storage: &B, output_id: &OutputId) -> Result<bool, Error> {
    Exist::<Unspent, ()>::exist(storage, &Unspent::new(*output_id))
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}
