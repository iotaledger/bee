// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::consensus::storage::StorageBackend as LedgerStorageBackend;
use bee_protocol::storage::StorageBackend as ProtocolStorageBackend;
use bee_rest_api::endpoints::storage::StorageBackend as RestApiStorageBackend;
use bee_snapshot::storage::StorageBackend as SnapshotStorageBackend;
use bee_storage::backend;
use bee_tangle::storage::StorageBackend as TangleStorageBackend;

pub trait StorageBackend:
    backend::StorageBackend
    + LedgerStorageBackend
    + ProtocolStorageBackend
    + RestApiStorageBackend
    + SnapshotStorageBackend
    + TangleStorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + LedgerStorageBackend
        + ProtocolStorageBackend
        + RestApiStorageBackend
        + SnapshotStorageBackend
        + TangleStorageBackend
{
}
