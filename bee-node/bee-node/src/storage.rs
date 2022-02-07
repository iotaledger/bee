// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::workers::storage::StorageBackend as LedgerStorageBackend;
use bee_protocol::workers::storage::StorageBackend as ProtocolStorageBackend;
use bee_rest_api::endpoints::storage::StorageBackend as RestApiStorageBackend;
use bee_tangle::storage::StorageBackend as TangleStorageBackend;

/// Node storage operations.
pub trait NodeStorageBackend:
    bee_storage::backend::StorageBackend
    + LedgerStorageBackend
    + ProtocolStorageBackend
    + RestApiStorageBackend
    + TangleStorageBackend
{
}

impl<T> NodeStorageBackend for T where
    T: bee_storage::backend::StorageBackend
        + LedgerStorageBackend
        + ProtocolStorageBackend
        + RestApiStorageBackend
        + TangleStorageBackend
{
}
