// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::info::SnapshotInfo;

use bee_ledger::model::{LedgerIndex, Output, Unspent};
use bee_message::payload::transaction::{Ed25519Address, OutputId};
use bee_storage::{
    access::{Fetch, Insert, Truncate},
    backend,
};
use bee_tangle::{milestone::MilestoneIndex, solid_entry_point::SolidEntryPoint};

pub trait StorageBackend:
    backend::StorageBackend
    + Fetch<(), SnapshotInfo>
    + Insert<SolidEntryPoint, MilestoneIndex>
    + Insert<(), LedgerIndex>
    + Insert<OutputId, Output>
    + Insert<Unspent, ()>
    + Insert<(Ed25519Address, OutputId), ()>
    + Truncate<SolidEntryPoint, MilestoneIndex>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Fetch<(), SnapshotInfo>
        + Insert<SolidEntryPoint, MilestoneIndex>
        + Insert<(), LedgerIndex>
        + Insert<OutputId, Output>
        + Insert<Unspent, ()>
        + Insert<(Ed25519Address, OutputId), ()>
        + Truncate<SolidEntryPoint, MilestoneIndex>
{
}
