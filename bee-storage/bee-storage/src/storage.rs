// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use serde::de::DeserializeOwned;

use std::error::Error;

#[async_trait]
/// Trait to be implemented on storage backend, which determine how to start and shutdown the storage.
pub trait Backend: Sized + Send + Sync + 'static {
    type ConfigBuilder: Default + DeserializeOwned + Into<Self::Config>;
    type Config: Clone + Send + Sync;
    type Error: std::error::Error + Send;

    /// Start method should impl how to start and initialize the corrsponding database.
    /// It takes config_path which define the database options, and returns Result<Self, Box<dyn Error>>.
    async fn start(config: Self::Config) -> Result<Self, Box<dyn Error>>;

    /// Shutdown method should impl how to shutdown the corrsponding database.
    /// It takes the ownership of self, and returns () or error.
    async fn shutdown(self) -> Result<(), Box<dyn Error>>;
}
