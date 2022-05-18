// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! This module forms the backend layer which holds the contracts of starting and shutting down the backend, as well as
//! accessing backend properties.

use serde::de::DeserializeOwned;

use crate::{
    access::{
        AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, InsertStrict, MultiFetch, Truncate, Update,
    },
    system::StorageHealth,
};

/// Trait to be implemented on a storage backend.
/// Determines how to start and shutdown the backend.
pub trait StorageBackend: Send + Sized + Sync + 'static {
    /// Helps build the associated `Config`.
    type ConfigBuilder: Default + DeserializeOwned + Into<Self::Config>;
    /// Holds the backend options.
    type Config: Clone + Send + Sync;
    /// Returned on failed operations.
    type Error: std::error::Error + Send;

    /// Initializes and starts the backend.
    fn start(config: Self::Config) -> Result<Self, Self::Error>;

    /// Shutdowns the backend.
    fn shutdown(self) -> Result<(), Self::Error>;

    /// Returns the size of the database in bytes.
    /// Not all backends may be able to provide this operation.
    fn size(&self) -> Result<Option<usize>, Self::Error>;

    /// Returns the health status of the database.
    /// Not all backends may be able to provide this operation.
    fn get_health(&self) -> Result<Option<StorageHealth>, Self::Error>;

    /// Sets the health status of the database.
    /// Not all backends may be able to provide this operation.
    fn set_health(&self, health: StorageHealth) -> Result<(), Self::Error>;
}

/// Convenience trait to simplify access operations.
pub trait StorageBackendExt: StorageBackend {
    /// Add Insert batch operation for the provided key value pair into the Batch memory buffer.
    fn batch_insert<K, V>(&self, batch: &mut Self::Batch, key: &K, value: &V) -> Result<(), Self::Error>
    where
        Self: Batch<K, V>;

    /// Add Delete batch operation for the provided key value pair into the Batch memory buffer.
    fn batch_delete<K, V>(&self, batch: &mut Self::Batch, key: &K) -> Result<(), Self::Error>
    where
        Self: Batch<K, V>;

    /// Deletes the value associated with the key from the storage.
    fn delete<K, V>(&self, key: &K) -> Result<(), Self::Error>
    where
        Self: Delete<K, V>;

    /// Checks if a value exists in the storage for the given key.
    fn exist<K, V>(&self, key: &K) -> Result<bool, Self::Error>
    where
        Self: Exist<K, V>;

    /// Fetches the value associated with the key from the storage.
    fn fetch_access<K, V>(&self, key: &K) -> Result<Option<V>, Self::Error>
    where
        Self: Fetch<K, V>;

    /// Inserts the (K, V) pair in the storage overwriting the value if it already exists.
    fn insert<K, V>(&self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        Self: Insert<K, V>;

    /// Inserts the (K, V) pair in the storage without overwriting the value if it already exists.
    fn insert_strict<K, V>(&self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        Self: InsertStrict<K, V>;

    /// Returns a `Iterator` object for the provided <K, V> collection.
    fn iter<'a, K, V>(&'a self) -> Result<Self::AsIter, Self::Error>
    where
        Self: AsIterator<'a, K, V>;

    /// Fetches the values associated with the keys from the storage.
    fn multi_fetch<'a, K, V>(&'a self, keys: &'a [K]) -> Result<Self::Iter, Self::Error>
    where
        Self: MultiFetch<'a, K, V>;

    /// Truncates all the entries associated with the (K, V) pair from the storage.
    fn truncate<K, V>(&self) -> Result<(), Self::Error>
    where
        Self: Truncate<K, V>;

    /// Fetches the value for the key `K` and updates it using `f`.
    fn update<K, V>(&self, key: &K, f: impl FnMut(&mut V)) -> Result<(), Self::Error>
    where
        Self: Update<K, V>;
}

impl<S: StorageBackend> StorageBackendExt for S {
    #[inline(always)]
    fn batch_insert<K, V>(
        &self,
        batch: &mut <Self as BatchBuilder>::Batch,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        Self: Batch<K, V>,
    {
        Batch::<K, V>::batch_insert(self, batch, key, value)
    }

    #[inline(always)]
    fn batch_delete<K, V>(&self, batch: &mut <Self as BatchBuilder>::Batch, key: &K) -> Result<(), Self::Error>
    where
        Self: Batch<K, V>,
    {
        Batch::<K, V>::batch_delete(self, batch, key)
    }

    #[inline(always)]
    fn delete<K, V>(&self, key: &K) -> Result<(), Self::Error>
    where
        Self: Delete<K, V>,
    {
        Delete::<K, V>::delete(self, key)
    }

    #[inline(always)]
    fn exist<K, V>(&self, key: &K) -> Result<bool, Self::Error>
    where
        Self: Exist<K, V>,
    {
        Exist::<K, V>::exist(self, key)
    }

    #[inline(always)]
    fn fetch_access<K, V>(&self, key: &K) -> Result<Option<V>, Self::Error>
    where
        Self: Fetch<K, V>,
    {
        Fetch::<K, V>::fetch(self, key)
    }

    #[inline(always)]
    fn insert<K, V>(&self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        Self: Insert<K, V>,
    {
        Insert::<K, V>::insert(self, key, value)
    }

    #[inline(always)]
    fn insert_strict<K, V>(&self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        Self: InsertStrict<K, V>,
    {
        InsertStrict::<K, V>::insert_strict(self, key, value)
    }

    #[inline(always)]
    fn iter<'a, K, V>(&'a self) -> Result<<Self as AsIterator<'a, K, V>>::AsIter, Self::Error>
    where
        Self: AsIterator<'a, K, V>,
    {
        AsIterator::<'a, K, V>::iter(self)
    }

    #[inline(always)]
    fn multi_fetch<'a, K, V>(&'a self, keys: &'a [K]) -> Result<<Self as MultiFetch<'a, K, V>>::Iter, Self::Error>
    where
        Self: MultiFetch<'a, K, V>,
    {
        MultiFetch::<'a, K, V>::multi_fetch(self, keys)
    }

    #[inline(always)]
    fn truncate<K, V>(&self) -> Result<(), Self::Error>
    where
        Self: Truncate<K, V>,
    {
        Truncate::<K, V>::truncate(self)
    }

    #[inline(always)]
    fn update<K, V>(&self, key: &K, f: impl FnMut(&mut V)) -> Result<(), Self::Error>
    where
        Self: Update<K, V>,
    {
        Update::<K, V>::update(self, key, f)
    }
}
