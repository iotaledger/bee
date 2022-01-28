// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::{ConsumedOutput, OutputDiff, Receipt};
use bee_message::{
    address::Ed25519Address, milestone::MilestoneIndex, output::OutputId, payload::indexation::PaddedIndex, MessageId,
};
use bee_storage::{
    access::{AsIterator, Fetch},
    backend,
};

pub trait StorageBackend:
    backend::StorageBackend
    + Fetch<PaddedIndex, Vec<MessageId>>
    + Fetch<Ed25519Address, Vec<OutputId>>
    + Fetch<MilestoneIndex, OutputDiff>
    + Fetch<MilestoneIndex, Vec<Receipt>>
    + Fetch<OutputId, ConsumedOutput>
    + for<'a> AsIterator<'a, (MilestoneIndex, Receipt), ()>
    + bee_protocol::workers::storage::StorageBackend
    + bee_ledger::workers::storage::StorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Fetch<PaddedIndex, Vec<MessageId>>
        + Fetch<Ed25519Address, Vec<OutputId>>
        + Fetch<MilestoneIndex, OutputDiff>
        + Fetch<MilestoneIndex, Vec<Receipt>>
        + Fetch<OutputId, ConsumedOutput>
        + for<'a> AsIterator<'a, (MilestoneIndex, Receipt), ()>
        + bee_protocol::workers::storage::StorageBackend
        + bee_ledger::workers::storage::StorageBackend
{
}
