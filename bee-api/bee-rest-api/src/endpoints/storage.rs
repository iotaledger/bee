// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::{ConsumedOutput, OutputDiff, Receipt};
use bee_message::{
    address::Ed25519Address,
    output::OutputId,
    payload::{
        milestone::{MilestoneId, MilestoneIndex},
        MilestonePayload,
    },
};
use bee_storage::{
    access::{AsIterator, Fetch},
    backend,
};
use bee_tangle::milestone_metadata::MilestoneMetadata;

pub trait StorageBackend:
    backend::StorageBackend
    + Fetch<Ed25519Address, Vec<OutputId>>
    + Fetch<MilestoneIndex, OutputDiff>
    + Fetch<MilestoneIndex, Vec<Receipt>>
    + Fetch<OutputId, ConsumedOutput>
    + Fetch<MilestoneIndex, MilestoneMetadata>
    + Fetch<MilestoneId, MilestonePayload>
    + for<'a> AsIterator<'a, (MilestoneIndex, Receipt), ()>
    + bee_protocol::workers::storage::StorageBackend
    + bee_ledger::workers::storage::StorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Fetch<Ed25519Address, Vec<OutputId>>
        + Fetch<MilestoneIndex, OutputDiff>
        + Fetch<MilestoneIndex, Vec<Receipt>>
        + Fetch<OutputId, ConsumedOutput>
        + Fetch<MilestoneIndex, MilestoneMetadata>
        + Fetch<MilestoneId, MilestonePayload>
        + for<'a> AsIterator<'a, (MilestoneIndex, Receipt), ()>
        + bee_protocol::workers::storage::StorageBackend
        + bee_ledger::workers::storage::StorageBackend
{
}
