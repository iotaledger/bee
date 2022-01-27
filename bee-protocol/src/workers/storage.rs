// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::workers::StorageBackend as LedgerStorageBackend;
use bee_message::{milestone::MilestoneIndex, payload::indexation::PaddedIndex, MessageId};
use bee_storage::{access::Insert, backend};
use bee_tangle::unreferenced_message::UnreferencedMessage;

pub trait StorageBackend:
    backend::StorageBackend
    + Insert<(PaddedIndex, MessageId), ()>
    + Insert<(MilestoneIndex, UnreferencedMessage), ()>
    + LedgerStorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Insert<(PaddedIndex, MessageId), ()>
        + Insert<(MilestoneIndex, UnreferencedMessage), ()>
        + LedgerStorageBackend
{
}
