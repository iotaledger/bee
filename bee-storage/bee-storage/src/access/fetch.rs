// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::backend::StorageBackend;

/// `Fetch<K, V>` trait extends the `StorageBackend` with `fetch` operation for the (key: K, value: V pair);
/// therefore, it should be explicitly implemented for the corresponding `StorageBackend`.
#[async_trait::async_trait]
pub trait Fetch<K, V>: StorageBackend {
    /// Fetches the value associated with the key from the storage.
    async fn fetch(&self, key: &K) -> Result<Option<V>, Self::Error>;
}
