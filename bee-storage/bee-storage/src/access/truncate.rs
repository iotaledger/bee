// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::storage::Backend;

#[async_trait::async_trait]
pub trait Truncate<K, V>: Backend {
    async fn truncate(&self) -> Result<(), Self::Error>
    where
        Self: Sized;
}
