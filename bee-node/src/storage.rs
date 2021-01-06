// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::storage::StorageBackend as LedgerStorageBackend;
use bee_protocol::storage::StorageBackend as ProtocolStorageBackend;
use bee_rest_api::storage::StorageBackend as RestApiStorageBackend;
use bee_snapshot::storage::StorageBackend as SnapshotStorageBackend;
use bee_storage::backend;

pub trait StorageBackend:
    backend::StorageBackend + LedgerStorageBackend + RestApiStorageBackend + SnapshotStorageBackend + ProtocolStorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + LedgerStorageBackend
        + RestApiStorageBackend
        + SnapshotStorageBackend
        + ProtocolStorageBackend
{
}
