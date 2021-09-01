// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! This module forms the backend layer which holds the contracts of starting and shutting down the backend, as well as
//! accessing backend properties.

use crate::system::StorageHealth;

use serde::de::DeserializeOwned;

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
