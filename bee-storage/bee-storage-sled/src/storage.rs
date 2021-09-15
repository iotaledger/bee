// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The sled storage backend.

use crate::{
    access::insert::insert_u8_system,
    config::{AccessConfig, SledConfig, SledConfigBuilder},
    error::Error,
};

use bee_storage::{
    access::Fetch,
    system::{StorageHealth, StorageVersion, System, SYSTEM_HEALTH_KEY, SYSTEM_VERSION_KEY},
    StorageBackend,
};

pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion(0);

/// The sled database.
pub struct Storage {
    // TODO remove when fetch limits are starting to be used
    #[allow(dead_code)]
    pub(crate) access_config: AccessConfig,
    pub(crate) inner: sled::Db,
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

        Ok(Self {
            inner,
            access_config: config.access,
        })
    }
}

impl StorageBackend for Storage {
    type ConfigBuilder = SledConfigBuilder;
    type Config = SledConfig;
    type Error = Error;

    fn start(config: Self::Config) -> Result<Self, Self::Error> {
        let storage = Self::new(config)?;

        match Fetch::<u8, System>::fetch(&storage, &SYSTEM_VERSION_KEY)? {
            Some(System::Version(version)) => {
                if version != STORAGE_VERSION {
                    return Err(Error::VersionMismatch(version, STORAGE_VERSION));
                }
            }
            None => insert_u8_system(&storage, &SYSTEM_VERSION_KEY, &System::Version(STORAGE_VERSION))?,
            _ => panic!("Another system value was inserted on the version key."),
        }

        if let Some(health) = storage.health()? {
            if health != StorageHealth::Healthy {
                return Err(Self::Error::UnhealthyStorage(health));
            }
        }

        storage.set_health(StorageHealth::Idle)?;

        Ok(storage)
    }

    fn shutdown(self) -> Result<(), Self::Error> {
        self.set_health(StorageHealth::Healthy)?;
        self.inner.flush()?;
        Ok(())
    }

    fn size(&self) -> Result<Option<usize>, Self::Error> {
        Ok(Some(self.inner.size_on_disk()? as usize))
    }

    fn version(&self) -> Result<Option<StorageVersion>, Self::Error> {
        Ok(Some(STORAGE_VERSION))
    }

    fn health(&self) -> Result<Option<StorageHealth>, Self::Error> {
        Ok(match Fetch::<u8, System>::fetch(self, &SYSTEM_HEALTH_KEY)? {
            Some(System::Health(health)) => Some(health),
            None => None,
            _ => panic!("Another system value was inserted on the health key."),
        })
    }

    fn set_health(&self, health: StorageHealth) -> Result<(), Self::Error> {
        insert_u8_system(self, &SYSTEM_HEALTH_KEY, &System::Health(health))
    }
}
