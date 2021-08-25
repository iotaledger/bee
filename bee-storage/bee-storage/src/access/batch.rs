// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::backend::StorageBackend;

/// `BatchBuilder` trait extends the `StorageBackend` with batch builder functionality; therefore it should be
/// explicitly implemented for the corresponding `StorageBackend`.
pub trait BatchBuilder: StorageBackend {
    /// Type that acts like a memory buffer which queue all the write operations.
    type Batch: Default + Send + Sized;

    /// Creates and returns the constraint `Batch` object.
    fn batch_begin() -> Self::Batch {
        Self::Batch::default()
    }

    /// Takes ownership of a batch object in order to commit it to the backend.
    /// Durability argument determines if the batch needs to be logged into a write ahead log or not.
    fn batch_commit(&self, batch: Self::Batch, durability: bool) -> Result<(), Self::Error>;
}

/// `Batch<K, V>` trait extends the `StorageBackend` with batch operations for the (key: K, value: V) pair;
/// therefore, it should be explicitly implemented for the corresponding `StorageBackend`.
pub trait Batch<K, V>: BatchBuilder {
    /// Add Insert batch operation for the provided key value pair into the Batch memory buffer.
    fn batch_insert(&self, batch: &mut Self::Batch, key: &K, value: &V) -> Result<(), Self::Error>;
    /// Add Delete batch operation for the provided key value pair into the Batch memory buffer.
    fn batch_delete(&self, batch: &mut Self::Batch, key: &K) -> Result<(), Self::Error>;
}
