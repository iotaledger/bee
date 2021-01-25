// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::model::{Balance, Output, Spent};
use bee_message::{
    payload::transaction::{Address, Ed25519Address, OutputId},
    prelude::HashedIndex,
    MessageId,
};
use bee_storage::{access::Fetch, backend};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<OutputId, Spent>
    + Fetch<HashedIndex, Vec<MessageId>>
    + Fetch<OutputId, Output>
    + Fetch<OutputId, Spent>
    + Fetch<Ed25519Address, Vec<OutputId>>
    + Fetch<Address, Balance>
    + bee_protocol::storage::StorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<OutputId, Spent>
        + Fetch<HashedIndex, Vec<MessageId>>
        + Fetch<OutputId, Output>
        + Fetch<OutputId, Spent>
        + Fetch<Ed25519Address, Vec<OutputId>>
        + bee_protocol::storage::StorageBackend
{
}
