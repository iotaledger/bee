// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::StorageBackend as LedgerStorageBackend;
use bee_rest_api::storage::StorageBackend as RestApiStorageBackend;
use bee_storage::backend;
use bee_tangle::storage::StorageBackend as TangleStorageBackend;

pub trait StorageBackend:
    backend::StorageBackend + LedgerStorageBackend + RestApiStorageBackend + TangleStorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend + LedgerStorageBackend + RestApiStorageBackend + TangleStorageBackend
{
}
