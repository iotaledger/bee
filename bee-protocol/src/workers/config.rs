// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::milestone_key_range::MilestoneKeyRange;

use bee_message::milestone::MilestoneIndex;

use serde::Deserialize;

const DEFAULT_MINIMUM_POW_SCORE: f64 = 4000.0;
const DEFAULT_COO_PUBLIC_KEY_COUNT: usize = 2;
const DEFAULT_COO_PUBLIC_KEY_RANGES: [(&str, MilestoneIndex, MilestoneIndex); 0] = [];
const DEFAULT_MESSAGE_WORKER_CACHE: usize = 10000;
const DEFAULT_STATUS_INTERVAL: u64 = 10;
const DEFAULT_MILESTONE_SYNC_COUNT: u32 = 200;

#[derive(Default, Deserialize)]
struct ProtocolCoordinatorConfigBuilder {
    public_key_count: Option<usize>,
    public_key_ranges: Option<Vec<MilestoneKeyRange>>,
}

#[derive(Default, Deserialize)]
struct ProtocolWorkersConfigBuilder {
    message_worker_cache: Option<usize>,
    status_interval: Option<u64>,
    milestone_sync_count: Option<u32>,
}

/// Builder for a `ProtocolConfig`.
#[derive(Default, Deserialize)]
pub struct ProtocolConfigBuilder {
    minimum_pow_score: Option<f64>,
    coordinator: ProtocolCoordinatorConfigBuilder,
    workers: ProtocolWorkersConfigBuilder,
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

    /// Sets the message worker cache of the `ProtocolConfigBuilder`.
    pub fn message_worker_cache(mut self, message_worker_cache: usize) -> Self {
        self.workers.message_worker_cache.replace(message_worker_cache);
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
                message_worker_cache: self
                    .workers
                    .message_worker_cache
                    .unwrap_or(DEFAULT_MESSAGE_WORKER_CACHE),
                status_interval: self.workers.status_interval.unwrap_or(DEFAULT_STATUS_INTERVAL),
                milestone_sync_count: self
                    .workers
                    .milestone_sync_count
                    .unwrap_or(DEFAULT_MILESTONE_SYNC_COUNT),
            },
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
    pub(crate) message_worker_cache: usize,
    pub(crate) status_interval: u64,
    pub(crate) milestone_sync_count: u32,
}

/// Configuration for the protocol.
#[derive(Clone)]
pub struct ProtocolConfig {
    pub(crate) minimum_pow_score: f64,
    pub(crate) coordinator: ProtocolCoordinatorConfig,
    pub(crate) workers: ProtocolWorkersConfig,
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
}
