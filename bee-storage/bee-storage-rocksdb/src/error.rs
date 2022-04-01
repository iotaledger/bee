// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_storage::system::StorageHealth;
use thiserror::Error;

use crate::storage::StorageVersion;

#[derive(Debug, Error)]
pub enum Error {
    #[error("rocksdb internal error: {0}")]
    RocksDb(#[from] rocksdb::Error),
    #[error("unknown column family {0}")]
    UnknownColumnFamily(&'static str),
    #[error("storage version mismatch, {0:?} != {1:?}, remove storage folder and restart")]
    VersionMismatch(StorageVersion, StorageVersion),
    #[error("unhealthy storage: {0:?}, remove storage folder and restart")]
    UnhealthyStorage(StorageHealth),
}
