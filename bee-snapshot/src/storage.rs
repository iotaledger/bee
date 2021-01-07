// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::info::SnapshotInfo;

use bee_ledger::model::LedgerIndex;
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
    + Truncate<SolidEntryPoint, MilestoneIndex>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Fetch<(), SnapshotInfo>
        + Insert<SolidEntryPoint, MilestoneIndex>
        + Insert<(), LedgerIndex>
        + Truncate<SolidEntryPoint, MilestoneIndex>
{
}
