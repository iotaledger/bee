// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing pruning configuration.

use std::time::Duration;

use humanize_rs::{bytes, duration};
use serde::Deserialize;

const PRUNING_MILESTONES_ENABLED_DEFAULT: bool = true;
const PRUNING_RECEIPTS_ENABLED_DEFAULT: bool = false;
const PRUNING_BY_SIZE_ENABLED_DEFAULT: bool = true;
const MAX_MILESTONES_TO_KEEP_DEFAULT: u32 = 60480;
const THRESHOLD_PERCENTAGE_DEFAULT: f32 = 10.0;
const COOLDOWN_TIME_DEFAULT: &str = "5m";
const TARGET_SIZE_DEFAULT: &str = "30Gb";

/// Builder for a [`PruningConfig`].
#[derive(Default, Deserialize, PartialEq)]
#[must_use]
pub struct PruningConfigBuilder {
    milestones: Option<PruningMilestonesConfigBuilder>,
    receipts: Option<PruningReceiptsConfigBuilder>,
    #[serde(alias = "bySize")]
    by_size: Option<PruningBySizeConfigBuilder>,
}

impl PruningConfigBuilder {
    /// Creates a new [`PruningConfigBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// TODO
    pub fn milestones(mut self, builder: PruningMilestonesConfigBuilder) -> Self {
        self.milestones.replace(builder);
        self
    }

    /// TODO
    pub fn receipts(mut self, builder: PruningReceiptsConfigBuilder) -> Self {
        self.receipts.replace(builder);
        self
    }

    /// TODO
    pub fn by_size(mut self, builder: PruningBySizeConfigBuilder) -> Self {
        self.by_size.replace(builder);
        self
    }

    /// Finishes the builder into a [`PruningConfig`].
    #[must_use]
    pub fn finish(self) -> PruningConfig {
        PruningConfig {
            pruning_milestones: self.milestones.unwrap_or_default().finish(),
            pruning_receipts: self.receipts.unwrap_or_default().finish(),
            pruning_by_size: self.by_size.unwrap_or_default().finish(),
        }
    }
}

/// The pruning configuration.
#[derive(Clone)]
pub struct PruningConfig {
    pruning_milestones: PruningMilestonesConfig,
    pruning_receipts: PruningReceiptsConfig,
    pruning_by_size: PruningBySizeConfig,
}

impl PruningConfig {
    /// Returns a builder to create a [`PruningConfig`].
    pub fn build() -> PruningConfigBuilder {
        PruningConfigBuilder::new()
    }

    /// TODO
    pub fn pruning_milestones(&self) -> &PruningMilestonesConfig {
        &self.pruning_milestones
    }

    /// TODO
    pub fn pruning_receipts(&self) -> &PruningReceiptsConfig {
        &self.pruning_receipts
    }

    /// TODO
    pub fn pruning_by_size(&self) -> &PruningBySizeConfig {
        &self.pruning_by_size
    }
}

/// TODO
#[derive(Default, Deserialize, PartialEq)]
#[must_use]
pub struct PruningMilestonesConfigBuilder {
    enabled: Option<bool>,
    #[serde(alias = "maxMilestonesToKeep")]
    max_milestones_to_keep: Option<u32>,
}

impl PruningMilestonesConfigBuilder {
    /// TODO
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled.replace(enabled);
        self
    }

    /// TODO
    pub fn max_milestones_to_keep(mut self, max_milestones_to_keep: u32) -> Self {
        self.max_milestones_to_keep.replace(max_milestones_to_keep);
        self
    }

    /// TODO
    #[must_use]
    pub fn finish(self) -> PruningMilestonesConfig {
        PruningMilestonesConfig {
            enabled: self.enabled.unwrap_or(PRUNING_MILESTONES_ENABLED_DEFAULT),
            max_milestones_to_keep: self.max_milestones_to_keep.unwrap_or(MAX_MILESTONES_TO_KEEP_DEFAULT),
        }
    }
}

/// TODO
#[derive(Default, Deserialize, PartialEq)]
#[must_use]
pub struct PruningReceiptsConfigBuilder {
    enabled: Option<bool>,
}

impl PruningReceiptsConfigBuilder {
    /// TODO
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled.replace(enabled);
        self
    }

    /// TODO
    #[must_use]
    pub fn finish(self) -> PruningReceiptsConfig {
        PruningReceiptsConfig {
            enabled: self.enabled.unwrap_or(PRUNING_RECEIPTS_ENABLED_DEFAULT),
        }
    }
}

/// TODO
#[derive(Default, Deserialize, PartialEq)]
#[must_use]
pub struct PruningBySizeConfigBuilder {
    enabled: Option<bool>,
    #[serde(alias = "targetSize")]
    target_size: Option<String>,
    #[serde(alias = "thresholdPercentage")]
    threshold_percentage: Option<f32>,
    #[serde(alias = "cooldownTime")]
    cooldown_time: Option<String>,
}

impl PruningBySizeConfigBuilder {
    /// TODO
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled.replace(enabled);
        self
    }

    /// TODO
    pub fn target_size(mut self, target_size: String) -> Self {
        self.target_size.replace(target_size);
        self
    }

    /// TODO
    pub fn threshold_percentage(mut self, threshold_percentage: f32) -> Self {
        self.threshold_percentage.replace(threshold_percentage);
        self
    }

    /// TODO
    pub fn cooldown_time(mut self, cooldown_time: String) -> Self {
        self.cooldown_time.replace(cooldown_time);
        self
    }

    /// TODO
    #[must_use]
    pub fn finish(self) -> PruningBySizeConfig {
        let target_size = self.target_size.unwrap_or_else(|| TARGET_SIZE_DEFAULT.to_string());
        let target_size = target_size
            .parse::<bytes::Bytes>()
            .expect("parse human-readable pruning target size")
            .size();

        let cooldown_time = self.cooldown_time.unwrap_or_else(|| COOLDOWN_TIME_DEFAULT.to_string());
        let cooldown_time =
            duration::parse(cooldown_time.as_ref()).expect("parse human-readable pruning cooldown time");

        PruningBySizeConfig {
            enabled: self.enabled.unwrap_or(PRUNING_BY_SIZE_ENABLED_DEFAULT),
            target_size,
            threshold_percentage: self.threshold_percentage.unwrap_or(THRESHOLD_PERCENTAGE_DEFAULT),
            cooldown_time,
        }
    }
}

/// TODO
#[derive(Clone)]
pub struct PruningMilestonesConfig {
    enabled: bool,
    max_milestones_to_keep: u32,
}

impl PruningMilestonesConfig {
    /// TODO
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// TODO
    pub fn max_milestones_to_keep(&self) -> u32 {
        self.max_milestones_to_keep
    }
}

/// TODO
#[derive(Clone)]
pub struct PruningReceiptsConfig {
    enabled: bool,
}

impl PruningReceiptsConfig {
    /// TODO
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

/// TODO
#[derive(Clone)]
pub struct PruningBySizeConfig {
    enabled: bool,
    target_size: usize,
    threshold_percentage: f32,
    cooldown_time: Duration,
}

impl PruningBySizeConfig {
    /// TODO
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// TODO
    pub fn target_size(&self) -> usize {
        self.target_size
    }

    /// TODO
    pub fn threshold_percentage(&self) -> f32 {
        self.threshold_percentage
    }

    /// TODO
    pub fn cooldown_time(&self) -> Duration {
        self.cooldown_time
    }
}
