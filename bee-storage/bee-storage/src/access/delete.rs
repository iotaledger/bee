// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::backend::StorageBackend;

/// `Delete<K, V>` trait extends the `StorageBackend` with `delete` operation for the (key: K, value: V) pair;
/// therefore, it should be explicitly implemented for the corresponding `StorageBackend`.
pub trait Delete<K, V>: StorageBackend {
    /// Deletes the value associated with the key from the storage.
    fn delete(&self, key: &K) -> Result<(), Self::Error>;
}
