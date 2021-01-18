// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::backend::StorageBackend;

/// `Insert<K, V>` trait extends the `StorageBackend` with `insert` operation for the (key: K, value: V) pair;
/// therefore, it should be explicitly implemented for the corresponding `StorageBackend`.
#[async_trait::async_trait]
pub trait Insert<K, V>: StorageBackend {
    /// Inserts the (K, V) pair in the storage.
    async fn insert(&self, key: &K, value: &V) -> Result<(), Self::Error>;
}
