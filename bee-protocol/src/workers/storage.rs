// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::payload::milestone::MilestoneIndex;
use bee_ledger::workers::StorageBackend as LedgerStorageBackend;
use bee_storage::{access::Insert, backend};
use bee_tangle::unreferenced_block::UnreferencedBlock;

pub trait StorageBackend:
    backend::StorageBackend + Insert<(MilestoneIndex, UnreferencedBlock), ()> + LedgerStorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend + Insert<(MilestoneIndex, UnreferencedBlock), ()> + LedgerStorageBackend
{
}
