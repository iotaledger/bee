use bee_storage::storage::Backend;

use std::{convert::Infallible, error::Error};

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
impl Backend for DummyBackend {
    type ConfigBuilder = DummyConfigBuilder;
    type Config = DummyConfig;
    type Error = Infallible;

    async fn start(_: Self::Config) -> Result<Self, Box<dyn Error>> {
        Ok(DummyBackend)
    }

    async fn shutdown(self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
