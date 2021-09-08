// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module related to storage backend error handling;

use bee_storage::system::{StorageHealth, StorageVersion};

use thiserror::Error;

/// Error to be raised when a rocksdb ackend operation fails.
#[derive(Debug, Error)]
pub enum Error {
    #[error("RocksDb internal error: {0}")]
    RocksDb(#[from] rocksdb::Error),
    #[error("Unknown column family {0}")]
    UnknownColumnFamily(&'static str),
    #[error("Storage version mismatch, {0:?} != {1:?}, remove storage folder and restart")]
    VersionMismatch(StorageVersion, StorageVersion),
    #[error("Unhealthy storage: {0:?}, remove storage folder and restart")]
    UnhealthyStorage(StorageHealth),
}
