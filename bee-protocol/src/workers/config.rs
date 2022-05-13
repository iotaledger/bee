// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{
    output::{ByteCostConfig, ByteCostConfigBuilder},
    payload::milestone::MilestoneIndex,
};
use serde::Deserialize;

use crate::types::milestone_key_range::MilestoneKeyRange;

const DEFAULT_MINIMUM_POW_SCORE: f64 = 4000.0;
const DEFAULT_COO_PUBLIC_KEY_COUNT: usize = 2;
const DEFAULT_COO_PUBLIC_KEY_RANGES: [(&str, MilestoneIndex, MilestoneIndex); 0] = [];
const DEFAULT_MESSAGE_WORKER_CACHE: usize = 10000;
const DEFAULT_STATUS_INTERVAL: u64 = 10;
const DEFAULT_MILESTONE_SYNC_COUNT: u32 = 200;

#[derive(Default, Deserialize, PartialEq)]
#[must_use]
struct ProtocolCoordinatorConfigBuilder {
    #[serde(alias = "publicKeyCount")]
    public_key_count: Option<usize>,
    #[serde(alias = "publicKeyRanges")]
    public_key_ranges: Option<Vec<MilestoneKeyRange>>,
}

#[derive(Default, Deserialize, PartialEq)]
#[must_use]
struct ProtocolWorkersConfigBuilder {
    #[serde(alias = "blockWorkerCache")]
    block_worker_cache: Option<usize>,
    #[serde(alias = "statusInterval")]
    status_interval: Option<u64>,
    #[serde(alias = "milestoneSyncCount")]
    milestone_sync_count: Option<u32>,
}

/// Builder for a `ProtocolConfig`.
#[derive(Default, Deserialize, PartialEq)]
#[must_use]
pub struct ProtocolConfigBuilder {
    #[serde(alias = "minimumPowScore")]
    minimum_pow_score: Option<f64>,
    coordinator: ProtocolCoordinatorConfigBuilder,
    workers: ProtocolWorkersConfigBuilder,
    #[serde(alias = "byteCost")]
    byte_cost: ByteCostConfigBuilder,
}

impl ProtocolConfigBuilder {
    /// Creates a new `ProtocolConfigBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the minimum PoW score of the `ProtocolConfigBuilder`.
    pub fn minimum_pow_score(mut self, minimum_pow_score: f64) -> Self {
        self.minimum_pow_score.replace(minimum_pow_score);
        self
    }

    /// Sets the coordinator public key count of the `ProtocolConfigBuilder`.
    pub fn coo_public_key_count(mut self, coo_public_key_count: usize) -> Self {
        self.coordinator.public_key_count.replace(coo_public_key_count);
        self
    }

    /// Sets the coordinator public key ranges of the `ProtocolConfigBuilder`.
    pub fn coo_public_key_ranges(mut self, coo_public_key_ranges: Vec<MilestoneKeyRange>) -> Self {
        self.coordinator.public_key_ranges.replace(coo_public_key_ranges);
        self
    }

    /// Sets the block worker cache of the `ProtocolConfigBuilder`.
    pub fn block_worker_cache(mut self, block_worker_cache: usize) -> Self {
        self.workers.block_worker_cache.replace(block_worker_cache);
        self
    }

    /// Sets the status interval of the `ProtocolConfigBuilder`.
    pub fn status_interval(mut self, status_interval: u64) -> Self {
        self.workers.status_interval.replace(status_interval);
        self
    }

    /// Sets the milestone sync count of the `ProtocolConfigBuilder`.
    pub fn milestone_sync_count(mut self, milestone_sync_count: u32) -> Self {
        self.workers.milestone_sync_count.replace(milestone_sync_count);
        self
    }

    /// Finishes the `ProtocolConfigBuilder` into a `ProtocolConfig`.
    #[must_use]
    pub fn finish(self) -> ProtocolConfig {
        ProtocolConfig {
            minimum_pow_score: self.minimum_pow_score.unwrap_or(DEFAULT_MINIMUM_POW_SCORE),
            coordinator: ProtocolCoordinatorConfig {
                public_key_count: self
                    .coordinator
                    .public_key_count
                    .unwrap_or(DEFAULT_COO_PUBLIC_KEY_COUNT),
                public_key_ranges: self.coordinator.public_key_ranges.unwrap_or_else(|| {
                    DEFAULT_COO_PUBLIC_KEY_RANGES
                        .iter()
                        .map(|(public_key, start, end)| MilestoneKeyRange::new(public_key.to_string(), *start, *end))
                        .collect()
                }),
            },
            workers: ProtocolWorkersConfig {
                block_worker_cache: self
                    .workers
                    .block_worker_cache
                    .unwrap_or(DEFAULT_MESSAGE_WORKER_CACHE),
                status_interval: self.workers.status_interval.unwrap_or(DEFAULT_STATUS_INTERVAL),
                milestone_sync_count: self
                    .workers
                    .milestone_sync_count
                    .unwrap_or(DEFAULT_MILESTONE_SYNC_COUNT),
            },
            byte_cost: self.byte_cost.finish(),
        }
    }
}

/// Configuration for the coordinator.
#[derive(Clone)]
pub struct ProtocolCoordinatorConfig {
    pub(crate) public_key_count: usize,
    pub(crate) public_key_ranges: Vec<MilestoneKeyRange>,
}

/// Configuration for the protocol workers.
#[derive(Clone)]
pub struct ProtocolWorkersConfig {
    pub(crate) block_worker_cache: usize,
    pub(crate) status_interval: u64,
    pub(crate) milestone_sync_count: u32,
}

/// Configuration for the protocol.
#[derive(Clone)]
pub struct ProtocolConfig {
    pub(crate) minimum_pow_score: f64,
    pub(crate) coordinator: ProtocolCoordinatorConfig,
    pub(crate) workers: ProtocolWorkersConfig,
    pub(crate) byte_cost: ByteCostConfig,
}

impl ProtocolConfig {
    /// Creates a new `ProtocolConfigBuilder`.
    pub fn build() -> ProtocolConfigBuilder {
        ProtocolConfigBuilder::new()
    }

    /// Returns the minimum PoW score of the `ProtocolConfig`.
    pub fn minimum_pow_score(&self) -> f64 {
        self.minimum_pow_score
    }

    /// Returns the coordinator configuration of the `ProtocolConfig`.
    pub fn coordinator(&self) -> &ProtocolCoordinatorConfig {
        &self.coordinator
    }

    /// Returns the byte cost configuration of the `ProtocolConfig`.
    pub fn byte_cost(&self) -> &ByteCostConfig {
        &self.byte_cost
    }
}
