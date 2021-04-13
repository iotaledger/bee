// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

const DEFAULT_ENABLED: bool = true;
const DEFAULT_DELAY: u32 = 60480;
const DEFAULT_PRUNE_RECEIPTS: bool = false;
const DEFAULT_BATCH_SIZE: usize = 1;

const MIN_BATCH_SIZE: usize = 1;

#[derive(Default, Deserialize)]
pub struct PruningConfigBuilder {
    enabled: Option<bool>,
    delay: Option<u32>,
    prune_receipts: Option<bool>,
    batch_size: Option<usize>,
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

    pub fn prune_receipts(mut self, prune_receipts: bool) -> Self {
        self.prune_receipts.replace(prune_receipts);
        self
    }

    pub fn batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size.replace(batch_size.max(MIN_BATCH_SIZE));
        self
    }

    pub fn finish(self) -> PruningConfig {
        PruningConfig {
            enabled: self.enabled.unwrap_or(DEFAULT_ENABLED),
            delay: self.delay.unwrap_or(DEFAULT_DELAY),
            prune_receipts: self.prune_receipts.unwrap_or(DEFAULT_PRUNE_RECEIPTS),
            batch_size: self.batch_size.unwrap_or(DEFAULT_BATCH_SIZE),
        }
    }
}

#[derive(Clone)]
pub struct PruningConfig {
    enabled: bool,
    delay: u32,
    prune_receipts: bool,
    batch_size: usize,
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

    pub fn prune_receipts(&self) -> bool {
        self.prune_receipts
    }

    pub fn batch_size(&self) -> usize {
        self.batch_size
    }
}
