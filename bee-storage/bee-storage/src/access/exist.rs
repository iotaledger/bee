// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::storage::Backend;

/// Exist<K, V> trait will extend the Backend with Exist operation for the key: K value: V pair
/// therefore it should be explicitly implemented for the corresponding Backend.
#[async_trait::async_trait]
pub trait Exist<K, V>: Backend {
    /// Execute Exist query
    async fn exist(&self, key: &K) -> Result<bool, Self::Error>
    where
        Self: Sized;
}
