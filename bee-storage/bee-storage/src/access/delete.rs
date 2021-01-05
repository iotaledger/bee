// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::backend::StorageBackend;

/// Delete<K, V> trait will extend the StorageBackend with Delete operation for the key: K value: V pair
/// therefore it should be explicitly implemented for the corresponding StorageBackend.
#[async_trait::async_trait]
pub trait Delete<K, V>: StorageBackend {
    /// Execute Delete query
    async fn delete(&self, key: &K) -> Result<(), Self::Error>
    where
        Self: Sized;
}
