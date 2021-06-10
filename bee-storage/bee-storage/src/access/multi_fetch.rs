// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::backend::StorageBackend;

/// `MultiFetch<K, V>` trait extends the `StorageBackend` with `multi_fetch` operation for the (key: K, value: V pair);
/// therefore, it should be explicitly implemented for the corresponding `StorageBackend`.
#[allow(clippy::type_complexity)]
pub trait MultiFetch<K, V>: StorageBackend {
    /// Fetches the values associated with the keys from the storage.
    fn multi_fetch(&self, keys: &[K]) -> Result<Vec<Result<Option<V>, Self::Error>>, Self::Error>;
}
