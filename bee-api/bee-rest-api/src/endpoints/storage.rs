// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::{OutputDiff, Receipt};
use bee_message::{
    address::Ed25519Address, milestone::MilestoneIndex, output::OutputId, payload::indexation::HashedIndex, MessageId,
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
    + bee_protocol::workers::storage::StorageBackend
    + bee_ledger::consensus::storage::StorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Fetch<HashedIndex, Vec<MessageId>>
        + Fetch<Ed25519Address, Vec<OutputId>>
        + Fetch<MilestoneIndex, OutputDiff>
        + Fetch<MilestoneIndex, Vec<Receipt>>
        + for<'a> AsStream<'a, (MilestoneIndex, Receipt), ()>
        + bee_protocol::workers::storage::StorageBackend
        + bee_ledger::consensus::storage::StorageBackend
{
}
