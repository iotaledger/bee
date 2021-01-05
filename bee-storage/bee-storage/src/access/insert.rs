// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::backend::StorageBackend;

/// Insert<K, V> trait will extend the StorageBackend with Insert operation for the key: K value: V pair
/// therefore it should be explicitly implemented for the corresponding StorageBackend.
#[async_trait::async_trait]
pub trait Insert<K, V>: StorageBackend {
    /// Execute Insert query
    async fn insert(&self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        Self: Sized;
}
