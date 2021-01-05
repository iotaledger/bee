// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::storage::StorageBackend;

#[async_trait::async_trait]
pub trait Truncate<K, V>: StorageBackend {
    async fn truncate(&self) -> Result<(), Self::Error>
    where
        Self: Sized;
}
