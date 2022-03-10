// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::backend::StorageBackend;

/// `Update<K, V>` trait extends the `StorageBackend` with `update` operation for the (key: K, value: V) pair;
/// therefore, it should be explicitly implemented for the corresponding `StorageBackend`.
pub trait Update<K, V>: StorageBackend {
    /// Fetches the value for the key `K` and updates it using `f`
    fn update(&self, key: &K, f: impl FnMut(&mut V)) -> Result<(), Self::Error>;
}
