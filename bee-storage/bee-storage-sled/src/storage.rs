// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The sled storage backend.

use crate::config::{SledConfig, SledConfigBuilder};

use bee_storage::{
    access::{Fetch, Insert},
    backend::StorageBackend,
    health::StorageHealth,
    system::{StorageVersion, System, SYSTEM_HEALTH_KEY, SYSTEM_VERSION_KEY},
};

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
/// Error to be raised when a backend operation fails.
pub enum Error {
    /// A sled operation failed.
    #[error("Sled internal error: {0}")]
    Sled(#[from] sled::Error),
    /// There is a storage version mismatch between the storage folder and this version of the
    /// storage.
    #[error("Storage version mismatch, {0:?} != {1:?}, remove storage folder and restart")]
    VersionMismatch(StorageVersion, StorageVersion),
    /// The storage was not closed properly.
    #[error("Unhealthy storage: {0:?}, remove storage folder and restart")]
    UnhealthyStorage(StorageHealth),
}

pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion(1);

/// The sled database.
pub struct Storage {
    pub(crate) inner: sled::Db,
    pub(crate) config: SledConfig,
}

impl Storage {
    /// Create a new database from the provided configuration.
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
            .compression_factor(config.compression_factor.unwrap_or(1) as i32)
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
        let storage = Self::new(config)?;

        match Fetch::<u8, System>::fetch(&storage, &SYSTEM_VERSION_KEY).await? {
            Some(System::Version(version)) => {
                if version != STORAGE_VERSION {
                    return Err(Error::VersionMismatch(version, STORAGE_VERSION));
                }
            }
            None => {
                Insert::<u8, System>::insert(&storage, &SYSTEM_VERSION_KEY, &System::Version(STORAGE_VERSION)).await?
            }
            _ => panic!("Another system value was inserted on the version key."),
        }

        if let Some(health) = storage.get_health().await? {
            if health != StorageHealth::Healthy {
                return Err(Self::Error::UnhealthyStorage(health));
            }
        }

        storage.set_health(StorageHealth::Idle).await?;

        Ok(storage)
    }

    async fn shutdown(self) -> Result<(), Self::Error> {
        self.set_health(StorageHealth::Healthy).await?;
        self.inner.flush_async().await?;
        Ok(())
    }

    async fn size(&self) -> Result<Option<usize>, Self::Error> {
        Ok(Some(self.inner.size_on_disk()? as usize))
    }

    async fn get_health(&self) -> Result<Option<StorageHealth>, Self::Error> {
        Ok(match Fetch::<u8, System>::fetch(self, &SYSTEM_HEALTH_KEY).await? {
            Some(System::Health(health)) => Some(health),
            None => None,
            _ => panic!("Another system value was inserted on the health key."),
        })
    }

    async fn set_health(&self, health: StorageHealth) -> Result<(), Self::Error> {
        Insert::<u8, System>::insert(self, &SYSTEM_HEALTH_KEY, &System::Health(health)).await
    }
}
