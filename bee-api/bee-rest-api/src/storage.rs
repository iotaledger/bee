// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::model::Balance;
use bee_message::{
    payload::transaction::{Address, ConsumedOutput, CreatedOutput, Ed25519Address, OutputId},
    prelude::HashedIndex,
    MessageId,
};
use bee_storage::{access::Fetch, backend};

pub trait StorageBackend:
    backend::StorageBackend
    + Fetch<HashedIndex, Vec<MessageId>>
    + Fetch<OutputId, CreatedOutput>
    + Fetch<OutputId, ConsumedOutput>
    + Fetch<Ed25519Address, Vec<OutputId>>
    + Fetch<Address, Balance>
    + bee_protocol::storage::StorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Fetch<HashedIndex, Vec<MessageId>>
        + Fetch<OutputId, CreatedOutput>
        + Fetch<OutputId, ConsumedOutput>
        + Fetch<Ed25519Address, Vec<OutputId>>
        + bee_protocol::storage::StorageBackend
{
}
