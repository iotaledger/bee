// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::info::SnapshotInfo;

use bee_message::{ledger_index::LedgerIndex, milestone::MilestoneIndex, solid_entry_point::SolidEntryPoint};
use bee_storage::{
    access::{Fetch, Insert, Truncate},
    backend,
};

pub trait StorageBackend:
    backend::StorageBackend
    + Fetch<(), SnapshotInfo>
    + Fetch<(), LedgerIndex>
    + Insert<SolidEntryPoint, MilestoneIndex>
    + Insert<(), LedgerIndex>
    + Insert<(), SnapshotInfo>
    + Truncate<SolidEntryPoint, MilestoneIndex>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Fetch<(), SnapshotInfo>
        + Fetch<(), LedgerIndex>
        + Insert<SolidEntryPoint, MilestoneIndex>
        + Insert<(), LedgerIndex>
        + Insert<(), SnapshotInfo>
        + Truncate<SolidEntryPoint, MilestoneIndex>
{
}
