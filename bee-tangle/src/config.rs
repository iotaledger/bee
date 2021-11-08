// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

use std::num::NonZeroUsize;

const DEFAULT_BELOW_MAX_DEPTH: u32 = 15;
// SAFETY: initialised with a non-zero value.
const DEFAULT_NUM_PARTITIONS: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(16) };
const DEFAULT_MAX_EVICTION_RETRIES: usize = 10;

/// A builder type for a tangle configuration.
#[derive(Default, Deserialize)]
pub struct TangleConfigBuilder {
    below_max_depth: Option<u32>,
    num_partitions: Option<NonZeroUsize>,
    max_eviction_retries: Option<usize>,
}

impl TangleConfigBuilder {
    /// Create a new [`TangleConfigBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Finish building tangle configuration, to create a [`TangleConfig`].
    pub fn finish(self) -> TangleConfig {
        TangleConfig {
            below_max_depth: self.below_max_depth.unwrap_or(DEFAULT_BELOW_MAX_DEPTH),
            num_partitions: self.num_partitions.unwrap_or(DEFAULT_NUM_PARTITIONS),
            max_eviction_retries: self.max_eviction_retries.unwrap_or(DEFAULT_MAX_EVICTION_RETRIES),
        }
    }
}

/// The configuration state of a tangle.
#[derive(Clone)]
pub struct TangleConfig {
    below_max_depth: u32,
    num_partitions: NonZeroUsize,
    max_eviction_retries: usize,
}

impl TangleConfig {
    /// Begin building a new [`TangleConfig`].
    pub fn build() -> TangleConfigBuilder {
        TangleConfigBuilder::new()
    }

    /// Get the value of `below_max_depth`.
    pub fn below_max_depth(&self) -> u32 {
        self.below_max_depth
    }

    /// Get the value of `num_partitions`.
    pub fn num_partitions(&self) -> NonZeroUsize {
        self.num_partitions
    }

    /// Get the value of `max_eviction_retries`.
    pub fn max_eviction_retries(&self) -> usize {
        self.max_eviction_retries
    }
}
