// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::backend::StorageBackend;

/// `AsIterator<'a, K, V>` trait extends the `StorageBackend` with `iter` operation for the (key: K, value: V) pair;
/// therefore, it should be explicitly implemented for the corresponding `StorageBackend`.
pub trait AsIterator<'a, K, V>: StorageBackend {
    /// Type to iterate through the <K, V> collection.
    type AsIter: Iterator<Item = Result<(K, V), Self::Error>>;

    /// Returns a `Iterator` object for the provided <K, V> collection.
    fn iter(&'a self) -> Result<Self::AsIter, Self::Error>;
}
