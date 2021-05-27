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
        let sled_cfg = sled::Config::default()
            .path(&config.path)
            .cache_capacity(config.cache_capacity as u64)
            .mode(if config.fast_mode {
                sled::Mode::HighThroughput
            } else {
                sled::Mode::LowSpace
            })
            .use_compression(config.compression_factor.is_some())
            .compression_factor(config.compression_factor.unwrap_or(0) as i32)
            .temporary(config.temporary)
            .create_new(!config.create_new);

        let inner = sled_cfg.open()?;

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
