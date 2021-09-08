// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    column_families::*,
    config::{RocksDbConfig, RocksDbConfigBuilder, StorageConfig},
    error::Error,
};

pub use bee_storage::{
    access::{Fetch, Insert},
    backend::StorageBackend,
    system::{StorageHealth, StorageVersion, System, SYSTEM_HEALTH_KEY, SYSTEM_VERSION_KEY},
};

use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBCompactionStyle, DBCompressionType, Env, FlushOptions, Options, DB,
};

pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion(0);

pub struct Storage {
    // TODO remove when fetch limits are starting to be used
    #[allow(dead_code)]
    pub(crate) config: StorageConfig,
    pub(crate) inner: DB,
}

impl Storage {
    fn new(config: RocksDbConfig) -> Result<Self, Error> {
        let cf_system = ColumnFamilyDescriptor::new(CF_SYSTEM, Options::default());

        let cf_message_id_to_message = ColumnFamilyDescriptor::new(CF_MESSAGE_ID_TO_MESSAGE, Options::default());

        let mut opts = Options::default();
        opts.create_if_missing(config.create_if_missing);
        opts.create_missing_column_families(config.create_missing_column_families);
        if config.enable_statistics {
            opts.enable_statistics();
        }
        opts.increase_parallelism(config.increase_parallelism);
        opts.optimize_for_point_lookup(config.optimize_for_point_lookup);
        opts.optimize_level_style_compaction(config.optimize_level_style_compaction);
        opts.optimize_universal_style_compaction(config.optimize_universal_style_compaction);
        opts.set_advise_random_on_open(config.set_advise_random_on_open);
        opts.set_allow_concurrent_memtable_write(config.set_allow_concurrent_memtable_write);
        opts.set_allow_mmap_reads(config.set_allow_mmap_reads);
        opts.set_allow_mmap_writes(config.set_allow_mmap_writes);
        opts.set_atomic_flush(config.set_atomic_flush);
        opts.set_bytes_per_sync(config.set_bytes_per_sync);
        opts.set_compaction_readahead_size(config.set_compaction_readahead_size);
        opts.set_compaction_style(DBCompactionStyle::from(config.set_compaction_style));
        opts.set_max_write_buffer_number(config.set_max_write_buffer_number);
        opts.set_write_buffer_size(config.set_write_buffer_size);
        opts.set_db_write_buffer_size(config.set_db_write_buffer_size);
        opts.set_disable_auto_compactions(config.set_disable_auto_compactions);
        opts.set_compression_type(DBCompressionType::from(config.set_compression_type));
        opts.set_unordered_write(config.set_unordered_write);
        opts.set_use_direct_io_for_flush_and_compaction(config.set_use_direct_io_for_flush_and_compaction);

        let mut env = Env::default()?;
        env.set_background_threads(config.env.set_background_threads);
        env.set_high_priority_background_threads(config.env.set_high_priority_background_threads);
        opts.set_env(&env);

        let db = DB::open_cf_descriptors(&opts, config.path, vec![cf_system, cf_message_id_to_message])?;

        let mut flushopts = FlushOptions::new();
        flushopts.set_wait(true);
        db.flush_opt(&flushopts)?;
        // This can't fail since the `CF_SYSTEM` column family has just been created.
        db.flush_cf_opt(db.cf_handle(CF_SYSTEM).unwrap(), &flushopts)?;

        Ok(Storage {
            config: config.storage,
            inner: db,
        })
    }

    pub(crate) fn cf_handle(&self, cf_str: &'static str) -> Result<&ColumnFamily, Error> {
        self.inner.cf_handle(cf_str).ok_or(Error::UnknownColumnFamily(cf_str))
    }
}

impl StorageBackend for Storage {
    type ConfigBuilder = RocksDbConfigBuilder;
    type Config = RocksDbConfig;
    type Error = Error;

    fn start(config: Self::Config) -> Result<Self, Self::Error> {
        let storage = Self::new(config)?;

        match Fetch::<u8, System>::fetch(&storage, &SYSTEM_VERSION_KEY)? {
            Some(System::Version(version)) => {
                if version != STORAGE_VERSION {
                    return Err(Error::VersionMismatch(version, STORAGE_VERSION));
                }
            }
            None => Insert::<u8, System>::insert(&storage, &SYSTEM_VERSION_KEY, &System::Version(STORAGE_VERSION))?,
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

        Ok(self.inner.flush()?)
    }

    fn size(&self) -> Result<Option<usize>, Self::Error> {
        Ok(Some(
            self.inner.live_files()?.iter().fold(0, |acc, file| acc + file.size),
        ))
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
        Insert::<u8, System>::insert(self, &SYSTEM_HEALTH_KEY, &System::Health(health))
    }
}
