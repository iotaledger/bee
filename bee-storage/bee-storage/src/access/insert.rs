// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::backend::StorageBackend;

/// `Insert<K, V>` trait extends the `StorageBackend` with `insert` operation for the (key: K, value: V) pair;
/// therefore, it should be explicitly implemented for the corresponding `StorageBackend`.
pub trait Insert<K, V>: StorageBackend {
    /// Inserts the (K, V) pair in the storage overwriting the value if it already exists.
    fn insert(&self, key: &K, value: &V) -> Result<(), Self::Error>;
}

/// `InsertStrict<K, V>` trait extends the `StorageBackend` with `insert_strict` operation for the
/// (key: K, value: V) pair; therefore, it should be explicitly implemented for the corresponding
/// `StorageBackend`.
pub trait InsertStrict<K, V>: StorageBackend {
    /// Inserts the (K, V) pair in the storage without overwriting the value if it already exists.
    fn insert_strict(&self, key: &K, value: &V) -> Result<(), Self::Error>;
}
