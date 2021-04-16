// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::system::StorageVersion;

use bee_storage::health::StorageHealth;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("RocksDb error: {0}.")]
    RocksDb(#[from] rocksdb::Error),
    #[error("Unknown column family {0}.")]
    UnknownCf(&'static str),
    #[error("Storage version mismatch ({0:?} != {1:?}), remove the storage and restart.")]
    VersionMismatch(StorageVersion, StorageVersion),
    #[error("Unhealthy storage: {0:?}, remove storage folder and restart.")]
    UnhealthyStorage(StorageHealth),
}
