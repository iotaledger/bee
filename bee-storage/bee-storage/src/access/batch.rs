// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::storage::Backend;

#[async_trait::async_trait]
/// BatchBuilder trait will extend the Backend with Batch builder functionality,
/// therefore it should be explicitly implemented for the corresponding Backend.
pub trait BatchBuilder: Backend + Sized {
    /// Batch type acts like memory buffer which queue all the write operations.
    type Batch: Default + Send + Sized;

    /// This method will create and return the constraint Batch object
    fn batch_begin() -> Self::Batch {
        Self::Batch::default()
    }
    /// This method invoked through a backend reference
    /// It takes the ownership of a batch object, in order to commit it to the backend.
    /// Durability argument will determin if the batch needs to be logged into a write ahead log or not.
    /// It returns () which indicate successful commit operation
    async fn batch_commit(&self, batch: Self::Batch, durability: bool) -> Result<(), Self::Error>;
}

/// Batch<K, V> trait will extend the Backend with Batch operations for the key: K value: V pair
/// therefore it should be explicitly implemented for the corresponding Backend.
pub trait Batch<K, V>: Backend + BatchBuilder + Sized {
    /// Add Insert batch operation for the provided key value pair into the Batch memory buffer.
    fn batch_insert(&self, batch: &mut Self::Batch, key: &K, value: &V) -> Result<(), Self::Error>;
    /// Add Delete batch operation for the provided key value pair into the Batch memory buffer.
    fn batch_delete(&self, batch: &mut Self::Batch, key: &K) -> Result<(), Self::Error>;
}
