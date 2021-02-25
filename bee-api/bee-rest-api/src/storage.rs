// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::model::{OutputDiff, Receipt};
use bee_message::{
    address::{Address, Ed25519Address},
    output::{ConsumedOutput, CreatedOutput, OutputId},
    prelude::{HashedIndex, MilestoneIndex},
    milestone::MilestoneIndex,
    MessageId,
};
use bee_storage::{
    access::{AsStream, Fetch},
    backend,
};

pub trait StorageBackend:
    backend::StorageBackend
    + Fetch<HashedIndex, Vec<MessageId>>
    + Fetch<Ed25519Address, Vec<OutputId>>
    + Fetch<MilestoneIndex, OutputDiff>
    + Fetch<MilestoneIndex, Vec<Receipt>>
    + for<'a> AsStream<'a, (MilestoneIndex, Receipt), ()>
    + bee_protocol::storage::StorageBackend
    + bee_ledger::storage::StorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Fetch<HashedIndex, Vec<MessageId>>
        + Fetch<Ed25519Address, Vec<OutputId>>
        + Fetch<MilestoneIndex, OutputDiff>
        + Fetch<MilestoneIndex, Vec<Receipt>>
        + for<'a> AsStream<'a, (MilestoneIndex, Receipt), ()>
        + bee_protocol::storage::StorageBackend
        + bee_ledger::storage::StorageBackend
{
}
