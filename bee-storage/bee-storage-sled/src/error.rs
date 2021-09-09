// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module related to storage backend error handling;

use bee_storage::system::{StorageHealth, StorageVersion};

use thiserror::Error;

/// Error to be raised when a sled backend operation fails.
#[derive(Debug, Error)]
pub enum Error {
    /// A sled operation failed.
    #[error("Sled internal error: {0}")]
    Sled(#[from] sled::Error),
    /// There is a storage version mismatch between the storage folder and this version of the storage.
    #[error("Storage version mismatch, {0:?} != {1:?}, remove storage folder and restart")]
    VersionMismatch(StorageVersion, StorageVersion),
    /// The storage was not closed properly.
    #[error("Unhealthy storage: {0:?}, remove storage folder and restart")]
    UnhealthyStorage(StorageHealth),
}
