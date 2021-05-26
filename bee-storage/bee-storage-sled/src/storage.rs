// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{StorageConfig, StorageConfigBuilder},
    trees::*,
};

use bee_storage::{backend::StorageBackend, health::StorageHealth};

use async_trait::async_trait;
use sled;

type Error = sled::Error;

pub struct Storage {
    pub(crate) inner: sled::Db,
    pub(crate) config: StorageConfig,
}

impl Storage {
    pub fn new(config: StorageConfig) -> Result<Self, Error> {
        let inner = sled::open("./storage/mainnet")?;

        Ok(Self { inner, config })
    }
}

#[async_trait]
impl StorageBackend for Storage {
    type ConfigBuilder = StorageConfigBuilder;
    type Config = StorageConfig;
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
