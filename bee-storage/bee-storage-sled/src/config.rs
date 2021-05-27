// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

use std::path::PathBuf;

const DEFAULT_PATH: &str = "./storage/mainnet";
const DEFAULT_COMPRESSION_FACTOR: Option<usize> = None;
const DEFAULT_CACHE_CAPACITY: usize = 1_024 * 1_024 * 1_024;
const DEFAULT_FAST_MODE: bool = false;
const DEFAULT_TEMPORARY: bool = false;
const DEFAULT_CREATE_NEW: bool = false;
const DEFAULT_FETCH_EDGE_LIMIT: usize = 1_000;
const DEFAULT_FETCH_INDEX_LIMIT: usize = 1_000;
const DEFAULT_FETCH_OUTPUT_ID_LIMIT: usize = 1_000;
const DEFAULT_ITERATION_BUDGET: usize = 100;

#[derive(Clone)]
pub struct SledConfig {
    pub(crate) storage: StorageConfig,
    pub(crate) path: PathBuf,
    pub(crate) compression_factor: Option<usize>,
    pub(crate) cache_capacity: usize,
    pub(crate) fast_mode: bool,
    pub(crate) temporary: bool,
    pub(crate) create_new: bool,
}

#[derive(Default, Deserialize)]
pub struct SledConfigBuilder {
    storage: Option<StorageConfigBuilder>,
    path: Option<PathBuf>,
    compression_factor: Option<Option<usize>>,
    cache_capacity: Option<usize>,
    fast_mode: Option<bool>,
    temporary: Option<bool>,
    create_new: Option<bool>,
}

impl SledConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn with_compression_factor(mut self, compression_factor: Option<usize>) -> Self {
        self.compression_factor = Some(compression_factor);
        self
    }

    pub fn with_cache_capacity(mut self, cache_capacity: usize) -> Self {
        self.cache_capacity = Some(cache_capacity);
        self
    }

    pub fn with_mode(mut self, fast: bool) -> Self {
        self.fast_mode = Some(fast);
        self
    }

    pub fn with_temporary(mut self, temporary: bool) -> Self {
        self.temporary = Some(temporary);
        self
    }

    pub fn with_create_new(mut self, create_new: bool) -> Self {
        self.create_new = Some(create_new);
        self
    }

    pub fn finish(self) -> SledConfig {
        SledConfig {
            storage: self.storage.unwrap_or_default().finish(),
            path: self.path.unwrap_or_else(|| DEFAULT_PATH.into()),
            compression_factor: self.compression_factor.unwrap_or(DEFAULT_COMPRESSION_FACTOR),
            cache_capacity: self.cache_capacity.unwrap_or(DEFAULT_CACHE_CAPACITY),
            fast_mode: self.fast_mode.unwrap_or(DEFAULT_FAST_MODE),
            temporary: self.temporary.unwrap_or(DEFAULT_TEMPORARY),
            create_new: self.create_new.unwrap_or(DEFAULT_CREATE_NEW),
        }
    }
}

impl From<SledConfigBuilder> for SledConfig {
    fn from(builder: SledConfigBuilder) -> Self {
        builder.finish()
    }
}

#[derive(Clone)]
pub struct StorageConfig {
    pub(crate) fetch_edge_limit: usize,
    pub(crate) fetch_index_limit: usize,
    pub(crate) fetch_output_id_limit: usize,
    pub(crate) iteration_budget: usize,
}

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
