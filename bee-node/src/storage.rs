// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::consensus::storage::StorageBackend as LedgerStorageBackend;
use bee_protocol::workers::storage::StorageBackend as ProtocolStorageBackend;
use bee_rest_api::endpoints::storage::StorageBackend as RestApiStorageBackend;
use bee_storage::backend;
use bee_tangle::storage::StorageBackend as TangleStorageBackend;

pub trait StorageBackend:
    backend::StorageBackend + LedgerStorageBackend + ProtocolStorageBackend + RestApiStorageBackend + TangleStorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + LedgerStorageBackend
        + ProtocolStorageBackend
        + RestApiStorageBackend
        + TangleStorageBackend
{
}
