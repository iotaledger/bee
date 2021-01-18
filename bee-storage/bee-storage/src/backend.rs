// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use serde::de::DeserializeOwned;

/// Trait to be implemented on a storage backend.
/// Determines how to start and shutdown the backend.
#[async_trait]
pub trait StorageBackend: Send + Sized + Sync + 'static {
    /// Helps build the associated `Config`.
    type ConfigBuilder: Default + DeserializeOwned + Into<Self::Config>;
    /// Holds the backend options.
    type Config: Clone + Send + Sync;
    /// Returned on failed operations.
    type Error: std::error::Error + Send;

    /// Initializes and starts the backend.
    async fn start(config: Self::Config) -> Result<Self, Self::Error>;

    /// Shutdowns the backend.
    async fn shutdown(self) -> Result<(), Self::Error>;

    /// Size of the database in bytes. Not all backends may be able to provide it, hence the option.
    async fn size(&self) -> Result<Option<usize>, Self::Error>;
}
