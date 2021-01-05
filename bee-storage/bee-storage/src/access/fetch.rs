// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::storage::Backend;

/// Fetch<K, V> trait will extend the Backend with Fetch operation for the key: K value: V pair
/// therefore it should be explicitly implemented for the corresponding Backend.
#[async_trait::async_trait]
pub trait Fetch<K, V>: Backend {
    /// Execute Fetch query
    async fn fetch(&self, key: &K) -> Result<Option<V>, Self::Error>
    where
        Self: Sized;
}
