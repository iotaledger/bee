// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_storage::backend::StorageBackend;

use std::convert::Infallible;

use async_trait::async_trait;
use serde::Deserialize;

pub struct DummyBackend;

#[derive(Clone)]
pub struct DummyConfig;

#[derive(Default, Deserialize)]
pub struct DummyConfigBuilder;

impl Into<DummyConfig> for DummyConfigBuilder {
    fn into(self) -> DummyConfig {
        DummyConfig
    }
}

#[async_trait]
impl StorageBackend for DummyBackend {
    type ConfigBuilder = DummyConfigBuilder;
    type Config = DummyConfig;
    type Error = Infallible;

    async fn start(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(DummyBackend)
    }

    async fn shutdown(self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn size(&self) -> Result<Option<usize>, Self::Error> {
        Ok(None)
    }
}
