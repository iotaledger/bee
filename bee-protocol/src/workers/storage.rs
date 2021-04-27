// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::{types::snapshot::SnapshotInfo, workers::storage::StorageBackend as LedgerStorageBackend};
use bee_message::{milestone::MilestoneIndex, payload::indexation::PaddedIndex, MessageId};
use bee_storage::{
    access::{Batch, Fetch, Insert},
    backend,
};
use bee_tangle::{storage::StorageBackend as TangleStorageBackend, unreferenced_message::UnreferencedMessage};

pub trait StorageBackend:
    backend::StorageBackend
    + Batch<(MilestoneIndex, UnreferencedMessage), ()>
    + Insert<(PaddedIndex, MessageId), ()>
    + Insert<(MilestoneIndex, UnreferencedMessage), ()>
    + Fetch<(), SnapshotInfo>
    + TangleStorageBackend
    + LedgerStorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Batch<(MilestoneIndex, UnreferencedMessage), ()>
        + Insert<(PaddedIndex, MessageId), ()>
        + Insert<(MilestoneIndex, UnreferencedMessage), ()>
        + Fetch<(), SnapshotInfo>
        + TangleStorageBackend
        + LedgerStorageBackend
{
}
