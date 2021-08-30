// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing pruning configuration.

use serde::Deserialize;

const DEFAULT_ENABLED: bool = true;
const DEFAULT_DELAY: u32 = 60480;
const DEFAULT_PRUNE_RECEIPTS: bool = false;

/// Builder for a [`PruningConfig`].
#[derive(Default, Deserialize)]
pub struct PruningConfigBuilder {
    enabled: Option<bool>,
    delay: Option<u32>,
    prune_receipts: Option<bool>,
}

impl PruningConfigBuilder {
    /// Creates a new [`PruningConfigBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables pruning.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled.replace(enabled);
        self
    }

    /// Sets the pruning delay.
    pub fn delay(mut self, delay: u32) -> Self {
        self.delay.replace(delay);
        self
    }

    /// Sets whether receipts should be pruned as well.
    pub fn prune_receipts(mut self, prune_receipts: bool) -> Self {
        self.prune_receipts.replace(prune_receipts);
        self
    }

    /// Finishes the builder into a [`PruningConfig`].
    pub fn finish(self) -> PruningConfig {
        PruningConfig {
            enabled: self.enabled.unwrap_or(DEFAULT_ENABLED),
            delay: self.delay.unwrap_or(DEFAULT_DELAY),
            prune_receipts: self.prune_receipts.unwrap_or(DEFAULT_PRUNE_RECEIPTS),
        }
    }
}

/// The pruning configuration.
#[derive(Clone)]
pub struct PruningConfig {
    enabled: bool,
    delay: u32,
    prune_receipts: bool,
}

impl PruningConfig {
    /// Returns a builder to create a [`PruningConfig`].
    pub fn build() -> PruningConfigBuilder {
        PruningConfigBuilder::new()
    }

    /// Returns whether pruning is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns whether pruning is disabled.
    pub fn disabled(&self) -> bool {
        !self.enabled
    }

    /// Returns the pruning delay.
    pub fn delay(&self) -> u32 {
        self.delay
    }

    /// Returns whether [`Receipt`](crate::types::Receipt)s are pruned.
    pub fn prune_receipts(&self) -> bool {
        self.prune_receipts
    }
}
