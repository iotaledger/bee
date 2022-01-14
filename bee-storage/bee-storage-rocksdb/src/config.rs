// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types related to the backend configuration.

use crate::{compaction::CompactionStyle, compression::CompressionType};

use serde::Deserialize;

use std::path::PathBuf;

const DEFAULT_PATH: &str = "./storage/mainnet";
const DEFAULT_CREATE_IF_MISSING: bool = true;
const DEFAULT_CREATE_MISSING_COLUMN_FAMILIES: bool = true;
const DEFAULT_ENABLE_STATISTICS: bool = false;
const DEFAULT_OPTIMIZE_FOR_POINT_LOOKUP: u64 = 67_108_864; // 64 MiB
const DEFAULT_OPTIMIZE_LEVEL_STYLE_COMPACTION: usize = 0;
const DEFAULT_OPTIMIZE_UNIVERSAL_STYLE_COMPACTION: usize = 0;
const DEFAULT_SET_ADVISE_RANDOM_ON_OPEN: bool = true;
const DEFAULT_SET_ALLOW_CONCURRENT_MEMTABLE_WRITE: bool = true;
const DEFAULT_SET_ALLOW_MMAP_READS: bool = false;
const DEFAULT_SET_ALLOW_MMAP_WRITES: bool = false;
const DEFAULT_SET_ATOMIC_FLUSH: bool = false;
const DEFAULT_SET_BYTES_PER_SYNC: u64 = 0;
const DEFAULT_SET_COMPACTION_READAHEAD_SIZE: usize = 0;
const DEFAULT_SET_COMPACTION_STYLE: CompactionStyle = CompactionStyle::Fifo;
const DEFAULT_SET_MAX_WRITE_BUFFER_NUMBER: i32 = 2;
const DEFAULT_SET_WRITE_BUFFER_SIZE: usize = 67_108_864; // 64 MiB
const DEFAULT_SET_DB_WRITE_BUFFER_SIZE: usize = 67_108_864; // 64 MiB
const DEFAULT_SET_DISABLE_AUTO_COMPACTIONS: bool = false;
const DEFAULT_SET_COMPRESSION_TYPE: CompressionType = CompressionType::None;
const DEFAULT_SET_UNORDERED_WRITE: bool = true;
const DEFAULT_SET_USE_DIRECT_IO_FOR_FLUSH_AND_COMPACTION: bool = true;

const DEFAULT_SET_HIGH_PRIORITY_BACKGROUND_THREADS: i32 = 2;

/// Builder for an [`AccessConfig`].
#[derive(Default, Deserialize)]
pub struct AccessConfigBuilder {}

impl AccessConfigBuilder {
    /// Creates a new [`AccessConfigBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Consumes a [`AccessConfigBuilder`] to create an [`AccessConfig`].
    pub fn finish(self) -> AccessConfig {
        AccessConfig {}
    }
}

/// Builder for a [`RocksDbEnvConfig`].
#[derive(Default, Deserialize)]
pub struct RocksDbEnvConfigBuilder {
    set_background_threads: Option<i32>,
    set_high_priority_background_threads: Option<i32>,
}

impl RocksDbEnvConfigBuilder {
    /// Creates a new [`RocksDbEnvConfigBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Consumes a [`RocksDbEnvConfigBuilder`] to create a [`RocksDbEnvConfig`].
    pub fn finish(self) -> RocksDbEnvConfig {
        RocksDbEnvConfig {
            set_background_threads: self.set_background_threads.unwrap_or(num_cpus::get() as i32),
            set_high_priority_background_threads: self
                .set_high_priority_background_threads
                .unwrap_or(DEFAULT_SET_HIGH_PRIORITY_BACKGROUND_THREADS),
        }
    }
}

/// Builder for a [`RocksDbConfig`].
#[derive(Default, Deserialize)]
pub struct RocksDbConfigBuilder {
    access: Option<AccessConfigBuilder>,
    path: Option<String>,
    create_if_missing: Option<bool>,
    create_missing_column_families: Option<bool>,
    enable_statistics: Option<bool>,
    increase_parallelism: Option<i32>,
    optimize_for_point_lookup: Option<u64>,
    optimize_level_style_compaction: Option<usize>,
    optimize_universal_style_compaction: Option<usize>,
    set_advise_random_on_open: Option<bool>,
    set_allow_concurrent_memtable_write: Option<bool>,
    set_allow_mmap_reads: Option<bool>,
    set_allow_mmap_writes: Option<bool>,
    set_atomic_flush: Option<bool>,
    set_bytes_per_sync: Option<u64>,
    set_compaction_readahead_size: Option<usize>,
    set_compaction_style: Option<CompactionStyle>,
    set_max_write_buffer_number: Option<i32>,
    set_write_buffer_size: Option<usize>,
    set_db_write_buffer_size: Option<usize>,
    set_disable_auto_compactions: Option<bool>,
    set_compression_type: Option<CompressionType>,
    set_unordered_write: Option<bool>,
    set_use_direct_io_for_flush_and_compaction: Option<bool>,
    env: Option<RocksDbEnvConfigBuilder>,
}

impl RocksDbConfigBuilder {
    /// Creates a new [`RocksDbConfigBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the path of a [`RocksDbConfigBuilder`].
    #[must_use]
    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    /// Consumes a [`RocksDbConfigBuilder`] to create a [`RocksDbConfig`].
    pub fn finish(self) -> RocksDbConfig {
        RocksDbConfig::from(self)
    }
}

impl From<RocksDbConfigBuilder> for RocksDbConfig {
    fn from(builder: RocksDbConfigBuilder) -> Self {
        RocksDbConfig {
            access: builder.access.unwrap_or_default().finish(),
            path: PathBuf::from(builder.path.unwrap_or_else(|| DEFAULT_PATH.to_string())),
            create_if_missing: builder.create_if_missing.unwrap_or(DEFAULT_CREATE_IF_MISSING),
            create_missing_column_families: builder
                .create_missing_column_families
                .unwrap_or(DEFAULT_CREATE_MISSING_COLUMN_FAMILIES),
            enable_statistics: builder.enable_statistics.unwrap_or(DEFAULT_ENABLE_STATISTICS),
            increase_parallelism: builder.increase_parallelism.unwrap_or(num_cpus::get() as i32),
            optimize_for_point_lookup: builder
                .optimize_for_point_lookup
                .unwrap_or(DEFAULT_OPTIMIZE_FOR_POINT_LOOKUP),
            optimize_level_style_compaction: builder
                .optimize_level_style_compaction
                .unwrap_or(DEFAULT_OPTIMIZE_LEVEL_STYLE_COMPACTION),
            optimize_universal_style_compaction: builder
                .optimize_universal_style_compaction
                .unwrap_or(DEFAULT_OPTIMIZE_UNIVERSAL_STYLE_COMPACTION),
            set_advise_random_on_open: builder
                .set_advise_random_on_open
                .unwrap_or(DEFAULT_SET_ADVISE_RANDOM_ON_OPEN),
            set_allow_concurrent_memtable_write: builder
                .set_allow_concurrent_memtable_write
                .unwrap_or(DEFAULT_SET_ALLOW_CONCURRENT_MEMTABLE_WRITE),
            set_allow_mmap_reads: builder.set_allow_mmap_reads.unwrap_or(DEFAULT_SET_ALLOW_MMAP_READS),
            set_allow_mmap_writes: builder.set_allow_mmap_writes.unwrap_or(DEFAULT_SET_ALLOW_MMAP_WRITES),
            set_atomic_flush: builder.set_atomic_flush.unwrap_or(DEFAULT_SET_ATOMIC_FLUSH),
            set_bytes_per_sync: builder.set_bytes_per_sync.unwrap_or(DEFAULT_SET_BYTES_PER_SYNC),
            set_compaction_readahead_size: builder
                .set_compaction_readahead_size
                .unwrap_or(DEFAULT_SET_COMPACTION_READAHEAD_SIZE),
            set_compaction_style: builder.set_compaction_style.unwrap_or(DEFAULT_SET_COMPACTION_STYLE),
            set_max_write_buffer_number: builder
                .set_max_write_buffer_number
                .unwrap_or(DEFAULT_SET_MAX_WRITE_BUFFER_NUMBER),
            set_write_buffer_size: builder.set_write_buffer_size.unwrap_or(DEFAULT_SET_WRITE_BUFFER_SIZE),
            set_db_write_buffer_size: builder
                .set_db_write_buffer_size
                .unwrap_or(DEFAULT_SET_DB_WRITE_BUFFER_SIZE),
            set_disable_auto_compactions: builder
                .set_disable_auto_compactions
                .unwrap_or(DEFAULT_SET_DISABLE_AUTO_COMPACTIONS),
            set_compression_type: builder.set_compression_type.unwrap_or(DEFAULT_SET_COMPRESSION_TYPE),
            set_unordered_write: builder.set_unordered_write.unwrap_or(DEFAULT_SET_UNORDERED_WRITE),
            set_use_direct_io_for_flush_and_compaction: builder
                .set_use_direct_io_for_flush_and_compaction
                .unwrap_or(DEFAULT_SET_USE_DIRECT_IO_FOR_FLUSH_AND_COMPACTION),
            env: builder.env.unwrap_or_default().finish(),
        }
    }
}

/// Configuration related to the access operations of the storage.
#[derive(Clone)]
pub struct AccessConfig {}

/// Configuration for rocksdb env fine tuning.
#[derive(Clone)]
pub struct RocksDbEnvConfig {
    pub(crate) set_background_threads: i32,
    pub(crate) set_high_priority_background_threads: i32,
}

/// Configuration for rocksdb fine tuning.
#[derive(Clone)]
pub struct RocksDbConfig {
    pub(crate) access: AccessConfig,
    pub(crate) path: PathBuf,
    pub(crate) create_if_missing: bool,
    pub(crate) create_missing_column_families: bool,
    pub(crate) enable_statistics: bool,
    pub(crate) increase_parallelism: i32,
    pub(crate) optimize_for_point_lookup: u64,
    pub(crate) optimize_level_style_compaction: usize,
    pub(crate) optimize_universal_style_compaction: usize,
    pub(crate) set_advise_random_on_open: bool,
    pub(crate) set_allow_concurrent_memtable_write: bool,
    pub(crate) set_allow_mmap_reads: bool,
    pub(crate) set_allow_mmap_writes: bool,
    pub(crate) set_atomic_flush: bool,
    pub(crate) set_bytes_per_sync: u64,
    pub(crate) set_compaction_readahead_size: usize,
    pub(crate) set_compaction_style: CompactionStyle,
    pub(crate) set_max_write_buffer_number: i32,
    pub(crate) set_write_buffer_size: usize,
    pub(crate) set_db_write_buffer_size: usize,
    pub(crate) set_disable_auto_compactions: bool,
    pub(crate) set_compression_type: CompressionType,
    pub(crate) set_unordered_write: bool,
    pub(crate) set_use_direct_io_for_flush_and_compaction: bool,
    pub(crate) env: RocksDbEnvConfig,
}
