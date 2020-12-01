// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, output::Output, spent::Spent, unspent::Unspent};

use bee_message::payload::transaction::OutputId;
use bee_storage::{
    access::{Batch, BatchBuilder, Delete, Exist, Fetch, Insert},
    storage,
};

pub trait Backend:
    storage::Backend
    + BatchBuilder
    + Batch<OutputId, Output>
    + Batch<OutputId, Spent>
    + Batch<Unspent, ()>
    + Delete<OutputId, Output>
    + Delete<OutputId, Spent>
    + Delete<Unspent, ()>
    + Exist<OutputId, Output>
    + Exist<OutputId, Spent>
    + Exist<Unspent, ()>
    + Fetch<OutputId, Output>
    + Fetch<OutputId, Spent>
    + Insert<OutputId, Output>
    + Insert<OutputId, Spent>
    + Insert<Unspent, ()>
{
}

impl<T> Backend for T where
    T: storage::Backend
        + BatchBuilder
        + Batch<OutputId, Output>
        + Batch<OutputId, Spent>
        + Batch<Unspent, ()>
        + Delete<OutputId, Output>
        + Delete<OutputId, Spent>
        + Delete<Unspent, ()>
        + Exist<OutputId, Output>
        + Exist<OutputId, Spent>
        + Exist<Unspent, ()>
        + Fetch<OutputId, Output>
        + Fetch<OutputId, Spent>
        + Insert<OutputId, Output>
        + Insert<OutputId, Spent>
        + Insert<Unspent, ()>
{
}

pub(crate) async fn is_output_unspent<B: Backend>(storage: &B, output_id: &OutputId) -> Result<bool, Error> {
    storage
        .exist(&Unspent::new(*output_id))
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}
