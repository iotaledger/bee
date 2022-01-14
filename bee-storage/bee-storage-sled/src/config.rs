// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types related to the backend configuration.

use serde::Deserialize;

use std::path::PathBuf;

const DEFAULT_PATH: &str = "./storage/mainnet";
const DEFAULT_COMPRESSION_FACTOR: Option<usize> = None;
const DEFAULT_CACHE_CAPACITY: usize = 1_024 * 1_024 * 1_024;
const DEFAULT_FAST_MODE: bool = false;
const DEFAULT_TEMPORARY: bool = false;
const DEFAULT_CREATE_NEW: bool = false;

/// Builder for a [`SledConfig`].
#[derive(Default, Deserialize)]
pub struct SledConfigBuilder {
    access: Option<AccessConfigBuilder>,
    path: Option<PathBuf>,
    compression_factor: Option<Option<usize>>,
    cache_capacity: Option<usize>,
    fast_mode: Option<bool>,
    temporary: Option<bool>,
    create_new: Option<bool>,
}

impl SledConfigBuilder {
    /// Create a new builder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the path where the database will be stored.
    #[must_use]
    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set the compression factor for zstd, it must be an integer between 1 and 22.
    /// Do not use compression if the factor is `None`,
    #[must_use]
    pub fn with_compression_factor(mut self, compression_factor: Option<usize>) -> Self {
        self.compression_factor = Some(compression_factor);
        self
    }

    /// Set the page cache maximum capacity in bytes.
    #[must_use]
    pub fn with_cache_capacity(mut self, cache_capacity: usize) -> Self {
        self.cache_capacity = Some(cache_capacity);
        self
    }

    /// Specify if the database should priorize speed (true) or size (false).
    #[must_use]
    pub fn with_mode(mut self, fast: bool) -> Self {
        self.fast_mode = Some(fast);
        self
    }

    /// Set the database to be deleted after `Storage` is dropped.
    #[must_use]
    pub fn with_temporary(mut self, temporary: bool) -> Self {
        self.temporary = Some(temporary);
        self
    }

    /// Specify if the database should be created from scratch and fail if the `path` is already used.
    #[must_use]
    pub fn with_create_new(mut self, create_new: bool) -> Self {
        self.create_new = Some(create_new);
        self
    }

    /// Consumes a [`SledConfigBuilder`] to create a [`SledConfig`].
    pub fn finish(self) -> SledConfig {
        SledConfig {
            access: self.access.unwrap_or_default().finish(),
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

/// Builder for an [`AccessConfig`].
#[derive(Default, Deserialize)]
pub struct AccessConfigBuilder {}

impl AccessConfigBuilder {
    /// Create a new builder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Consumes an [`AccessConfigBuilder`] to create an [`AccessConfig`].
    pub fn finish(self) -> AccessConfig {
        AccessConfig {}
    }
}

/// Configuration related to the access operations of the storage.
#[derive(Clone)]
pub struct AccessConfig {}

/// Configuration for the sled storage backend.
#[derive(Clone)]
pub struct SledConfig {
    pub(crate) access: AccessConfig,
    pub(crate) path: PathBuf,
    pub(crate) compression_factor: Option<usize>,
    pub(crate) cache_capacity: usize,
    pub(crate) fast_mode: bool,
    pub(crate) temporary: bool,
    pub(crate) create_new: bool,
}
