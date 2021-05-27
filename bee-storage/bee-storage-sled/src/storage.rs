// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::config::{SledConfig, SledConfigBuilder};

use bee_storage::{backend::StorageBackend, health::StorageHealth};

use async_trait::async_trait;

pub type Error = sled::Error;

pub struct Storage {
    pub(crate) inner: sled::Db,
    pub(crate) config: SledConfig,
}

impl Storage {
    pub fn new(config: SledConfig) -> Result<Self, Error> {
        let inner = sled::open(&config.path)?;

        Ok(Self { inner, config })
    }
}

#[async_trait]
impl StorageBackend for Storage {
    type ConfigBuilder = SledConfigBuilder;
    type Config = SledConfig;
    type Error = Error;

    async fn start(config: Self::Config) -> Result<Self, Self::Error> {
        Self::new(config)
    }

    async fn shutdown(self) -> Result<(), Self::Error> {
        self.inner.flush()?;
        Ok(())
    }

    async fn size(&self) -> Result<Option<usize>, Self::Error> {
        Ok(Some(self.inner.size_on_disk()? as usize))
    }

    async fn get_health(&self) -> Result<Option<StorageHealth>, Self::Error> {
        Ok(None)
    }

    async fn set_health(&self, health: StorageHealth) -> Result<(), Self::Error> {
        Ok(())
    }
}
