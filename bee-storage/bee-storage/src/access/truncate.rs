// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::StorageBackend;

/// `Truncate<K, V>` trait extends the `StorageBackend` with `truncate` operation for the (key: K, value: V) pair;
/// therefore, it should be explicitly implemented for the corresponding `StorageBackend`.
pub trait Truncate<K, V>: StorageBackend {
    /// Truncates all the entries associated with the (K, V) pair from the storage.
    fn truncate(&self) -> Result<(), Self::Error>;
}
