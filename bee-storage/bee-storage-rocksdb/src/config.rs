// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{compaction::CompactionStyle, compression::CompressionType};

use serde::Deserialize;

const DEFAULT_FETCH_EDGE_LIMIT: usize = 1000;
const DEFAULT_FETCH_INDEX_LIMIT: usize = 1000;
const DEFAULT_FETCH_OUTPUT_ID_LIMIT: usize = 1000;
const DEFAULT_ITERATION_BUDGET: usize = 100;

const DEFAULT_PATH: &str = "./database";
const DEFAULT_CREATE_IF_MISSING: bool = true;
const DEFAULT_CREATE_MISSING_COLUMN_FAMILIES: bool = true;
const DEFAULT_ENABLE_STATISTICS: bool = true;
const DEFAULT_OPTIMIZE_FOR_POINT_LOOKUP: u64 = 0;
const DEFAULT_OPTIMIZE_LEVEL_STYLE_COMPACTION: usize = 0;
const DEFAULT_OPTIMIZE_UNIVERSAL_STYLE_COMPACTION: usize = 0;
const DEFAULT_SET_ADVISE_RANDOM_ON_OPEN: bool = true;
const DEFAULT_SET_ALLOW_CONCURRENT_MEMTABLE_WRITE: bool = true;
const DEFAULT_SET_ALLOW_MMAP_READS: bool = false;
const DEFAULT_SET_ALLOW_MMAP_WRITES: bool = false;
const DEFAULT_SET_ATOMIC_FLUSH: bool = true;
const DEFAULT_SET_BYTES_PER_SYNC: u64 = 0;
const DEFAULT_SET_COMPACTION_READAHEAD_SIZE: usize = 0;
const DEFAULT_SET_COMPACTION_STYLE: CompactionStyle = CompactionStyle::Level;
const DEFAULT_SET_MAX_WRITE_BUFFER_NUMBER: i32 = 2;
const DEFAULT_SET_DISABLE_AUTO_COMPACTIONS: bool = false;
const DEFAULT_SET_COMPRESSION_TYPE: CompressionType = CompressionType::None;

#[derive(Default, Deserialize)]
pub struct StorageConfigBuilder {
    fetch_edge_limit: Option<usize>,
    fetch_index_limit: Option<usize>,
    fetch_output_id_limit: Option<usize>,
    iteration_budget: Option<usize>,
}

impl StorageConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> StorageConfig {
        StorageConfig {
            fetch_edge_limit: self.fetch_edge_limit.unwrap_or(DEFAULT_FETCH_EDGE_LIMIT),
            fetch_index_limit: self.fetch_index_limit.unwrap_or(DEFAULT_FETCH_INDEX_LIMIT),
            fetch_output_id_limit: self.fetch_output_id_limit.unwrap_or(DEFAULT_FETCH_OUTPUT_ID_LIMIT),
            iteration_budget: self.iteration_budget.unwrap_or(DEFAULT_ITERATION_BUDGET),
        }
    }
}

#[derive(Default, Deserialize)]
pub struct RocksDBConfigBuilder {
    storage: Option<StorageConfigBuilder>,
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
    set_disable_auto_compactions: Option<bool>,
    set_compression_type: Option<CompressionType>,
}

impl RocksDBConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> RocksDBConfig {
        RocksDBConfig::from(self)
    }
}

impl From<RocksDBConfigBuilder> for RocksDBConfig {
    fn from(builder: RocksDBConfigBuilder) -> Self {
        RocksDBConfig {
            storage: builder.storage.unwrap_or_default().finish(),
            path: builder.path.unwrap_or_else(|| DEFAULT_PATH.to_string()),
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
            set_disable_auto_compactions: builder
                .set_disable_auto_compactions
                .unwrap_or(DEFAULT_SET_DISABLE_AUTO_COMPACTIONS),
            set_compression_type: builder.set_compression_type.unwrap_or(DEFAULT_SET_COMPRESSION_TYPE),
        }
    }
}

#[derive(Clone)]
pub struct StorageConfig {
    pub(crate) fetch_edge_limit: usize,
    pub(crate) fetch_index_limit: usize,
    pub(crate) fetch_output_id_limit: usize,
    pub(crate) iteration_budget: usize,
}

#[derive(Clone)]
pub struct RocksDBConfig {
    pub(crate) storage: StorageConfig,
    pub(crate) path: String,
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
    pub(crate) set_disable_auto_compactions: bool,
    pub(crate) set_compression_type: CompressionType,
}
