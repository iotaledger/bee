// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::milestone::{key_range::KeyRange, MilestoneIndex};

use serde::Deserialize;

const DEFAULT_MINIMUM_POW_SCORE: f64 = 4000.0;
const DEFAULT_COO_PUBLIC_KEY_COUNT: usize = 2;
const DEFAULT_COO_PUBLIC_KEY_RANGES: [(&str, MilestoneIndex, MilestoneIndex); 2] = [
    (
        "ed3c3f1a319ff4e909cf2771d79fece0ac9bd9fd2ee49ea6c0885c9cb3b1248c",
        MilestoneIndex(0),
        MilestoneIndex(0),
    ),
    (
        "f6752f5f46a53364e2ee9c4d662d762a81efd51010282a75cd6bd03f28ef349c",
        MilestoneIndex(0),
        MilestoneIndex(0),
    ),
];
const DEFAULT_MESSAGE_WORKER_CACHE: usize = 10000;
const DEFAULT_STATUS_INTERVAL: u64 = 10;
const DEFAULT_HANDSHAKE_WINDOW: u64 = 10;
const DEFAULT_MS_SYNC_COUNT: u32 = 1;

#[derive(Default, Deserialize)]
struct ProtocolCoordinatorConfigBuilder {
    public_key_count: Option<usize>,
    public_key_ranges: Option<Vec<KeyRange>>,
}

#[derive(Default, Deserialize)]
struct ProtocolWorkersConfigBuilder {
    message_worker_cache: Option<usize>,
    status_interval: Option<u64>,
    ms_sync_count: Option<u32>,
}

#[derive(Default, Deserialize)]
pub struct ProtocolConfigBuilder {
    minimum_pow_score: Option<f64>,
    coordinator: ProtocolCoordinatorConfigBuilder,
    workers: ProtocolWorkersConfigBuilder,
    handshake_window: Option<u64>,
}

impl ProtocolConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn minimum_pow_score(mut self, minimum_pow_score: f64) -> Self {
        self.minimum_pow_score.replace(minimum_pow_score);
        self
    }

    pub fn coo_public_key_ranges(mut self, coo_public_key_ranges: Vec<KeyRange>) -> Self {
        self.coordinator.public_key_ranges.replace(coo_public_key_ranges);
        self
    }

    pub fn message_worker_cache(mut self, message_worker_cache: usize) -> Self {
        self.workers.message_worker_cache.replace(message_worker_cache);
        self
    }

    pub fn ms_sync_count(mut self, ms_sync_count: u32) -> Self {
        self.workers.ms_sync_count.replace(ms_sync_count);
        self
    }

    pub fn status_interval(mut self, status_interval: u64) -> Self {
        self.workers.status_interval.replace(status_interval);
        self
    }

    pub fn handshake_window(mut self, handshake_window: u64) -> Self {
        self.handshake_window.replace(handshake_window);
        self
    }

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
                        .map(|(public_key, start, end)| KeyRange::new(public_key.to_string(), *start, *end))
                        .collect()
                }),
            },
            workers: ProtocolWorkersConfig {
                message_worker_cache: self
                    .workers
                    .message_worker_cache
                    .unwrap_or(DEFAULT_MESSAGE_WORKER_CACHE),
                status_interval: self.workers.status_interval.unwrap_or(DEFAULT_STATUS_INTERVAL),
                ms_sync_count: self.workers.ms_sync_count.unwrap_or(DEFAULT_MS_SYNC_COUNT),
            },
            handshake_window: self.handshake_window.unwrap_or(DEFAULT_HANDSHAKE_WINDOW),
        }
    }
}

#[derive(Clone)]
pub struct ProtocolCoordinatorConfig {
    pub(crate) public_key_count: usize,
    pub(crate) public_key_ranges: Vec<KeyRange>,
}

#[derive(Clone)]
pub struct ProtocolWorkersConfig {
    pub(crate) message_worker_cache: usize,
    pub(crate) status_interval: u64,
    pub(crate) ms_sync_count: u32,
}

#[derive(Clone)]
pub struct ProtocolConfig {
    pub(crate) minimum_pow_score: f64,
    pub(crate) coordinator: ProtocolCoordinatorConfig,
    pub(crate) workers: ProtocolWorkersConfig,
    pub(crate) handshake_window: u64,
}

impl ProtocolConfig {
    pub fn build() -> ProtocolConfigBuilder {
        ProtocolConfigBuilder::new()
    }

    pub fn coordinator(&self) -> &ProtocolCoordinatorConfig {
        &self.coordinator
    }
}
