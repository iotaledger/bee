// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

/// Pruning is enabled by default.
const DEFAULT_ENABLED: bool = true;
/// Pruning by default happens for milestones `DEFAULT_DELAY` indexes ago.
const DEFAULT_DELAY: u32 = 60480;
/// Pruning by default happens every `DEFAULT_INTERVAL` indexes.
const DEFAULT_INTERVAL: u32 = 50;
/// Pruning can only happen in at least `MIN_INTERVAL` steps.
const MIN_INTERVAL: u32 = 1;
/// Receipts are not pruned by default.
const DEFAULT_PRUNE_RECEIPTS: bool = false;

#[derive(Default, Deserialize)]
pub struct PruningConfigBuilder {
    enabled: Option<bool>,
    delay: Option<u32>,
    interval: Option<u32>,
    prune_receipts: Option<bool>,
}

impl PruningConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled.replace(enabled);
        self
    }

    pub fn delay(mut self, delay: u32) -> Self {
        self.delay.replace(delay);
        self
    }

    pub fn interval(mut self, interval: u32) -> Self {
        // We do not allow any number below `MIN_INTERVAL`.
        self.interval.replace(interval.max(MIN_INTERVAL));
        self
    }

    pub fn prune_receipts(mut self, prune_receipts: bool) -> Self {
        self.prune_receipts.replace(prune_receipts);
        self
    }

    pub fn finish(self) -> PruningConfig {
        PruningConfig {
            enabled: self.enabled.unwrap_or(DEFAULT_ENABLED),
            delay: self.delay.unwrap_or(DEFAULT_DELAY),
            prune_receipts: self.prune_receipts.unwrap_or(DEFAULT_PRUNE_RECEIPTS),
            interval: self.interval.unwrap_or(DEFAULT_INTERVAL),
        }
    }
}

#[derive(Clone)]
pub struct PruningConfig {
    enabled: bool,
    delay: u32,
    interval: u32,
    prune_receipts: bool,
}

impl PruningConfig {
    pub fn build() -> PruningConfigBuilder {
        PruningConfigBuilder::new()
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn delay(&self) -> u32 {
        self.delay
    }

    pub fn interval(&self) -> u32 {
        self.interval
    }

    pub fn prune_receipts(&self) -> bool {
        self.prune_receipts
    }
}
