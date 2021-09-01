// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::backend::StorageBackend;

/// `Exist<K, V>` trait extends the `StorageBackend` with `exist` operation for the (key: K, value: V) pair;
/// therefore, it should be explicitly implemented for the corresponding `StorageBackend`.
pub trait Exist<K, V>: StorageBackend {
    /// Checks if a value exists in the storage for the given key.
    fn exist(&self, key: &K) -> Result<bool, Self::Error>;
}
