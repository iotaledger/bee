// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::{balance::Balance, model::OutputDiff};
use bee_message::{
    address::{Address, Ed25519Address},
    output::{ConsumedOutput, CreatedOutput, OutputId},
    prelude::{HashedIndex, MilestoneIndex},
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
    + Fetch<MilestoneIndex, OutputDiff>
    + bee_protocol::storage::StorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Fetch<HashedIndex, Vec<MessageId>>
        + Fetch<OutputId, CreatedOutput>
        + Fetch<OutputId, ConsumedOutput>
        + Fetch<Ed25519Address, Vec<OutputId>>
        + Fetch<MilestoneIndex, OutputDiff>
        + bee_protocol::storage::StorageBackend
{
}
