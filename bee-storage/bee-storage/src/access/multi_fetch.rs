// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::backend::StorageBackend;

/// `MultiFetch<K, V>` trait extends the `StorageBackend` with `multi_fetch` operation for the (key: K, value: V pair);
/// therefore, it should be explicitly implemented for the corresponding `StorageBackend`.
#[async_trait::async_trait]
pub trait MultiFetch<K, V>: StorageBackend {
    /// Fetches the values associated with the keys from the storage.
    async fn multi_fetch(&self, keys: &[K]) -> Result<Vec<Option<V>>, Self::Error>;
}
