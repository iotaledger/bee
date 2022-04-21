// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing pruning configuration.

use std::time::Duration;

use humanize_rs::{bytes, duration};
use serde::Deserialize;

const PRUNING_MILESTONES_ENABLED_DEFAULT: bool = true;
const PRUNING_SIZE_ENABLED_DEFAULT: bool = true;
const PRUNING_RECEIPTS_ENABLED_DEFAULT: bool = false;
const MAX_MILESTONES_TO_KEEP_DEFAULT: u32 = 60480;
pub(crate) const MAX_MILESTONES_TO_KEEP_MINIMUM: u32 = 50;
const THRESHOLD_PERCENTAGE_DEFAULT: f32 = 10.0;
const COOLDOWN_TIME_DEFAULT: &str = "5m";
const TARGET_SIZE_DEFAULT: &str = "30Gb";

/// Builder for a [`PruningConfig`].
#[derive(Default, Debug, Deserialize, PartialEq)]
#[must_use]
pub struct PruningConfigBuilder {
    milestones: Option<PruningMilestonesConfigBuilder>,
    size: Option<PruningSizeConfigBuilder>,
    receipts: Option<PruningReceiptsConfigBuilder>,
}

impl PruningConfigBuilder {
    /// Creates a new [`PruningConfigBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`PruningMilestonesConfigBuilder`].
    pub fn milestones(mut self, builder: PruningMilestonesConfigBuilder) -> Self {
        self.milestones.replace(builder);
        self
    }

    /// Sets the [`PruningSizeConfigBuilder`].
    pub fn size(mut self, builder: PruningSizeConfigBuilder) -> Self {
        self.size.replace(builder);
        self
    }

    /// Sets the [`PruningReceiptsConfigBuilder`].
    pub fn receipts(mut self, builder: PruningReceiptsConfigBuilder) -> Self {
        self.receipts.replace(builder);
        self
    }

    /// Finishes the builder into a [`PruningConfig`].
    #[must_use]
    pub fn finish(self) -> PruningConfig {
        PruningConfig {
            milestones: self.milestones.unwrap_or_default().finish(),
            receipts: self.receipts.unwrap_or_default().finish(),
            size: self.size.unwrap_or_default().finish(),
        }
    }
}

/// Builder for a [`PruningMilestonesConfig`].
#[derive(Default, Debug, Deserialize, PartialEq)]
#[must_use]
pub struct PruningMilestonesConfigBuilder {
    enabled: Option<bool>,
    #[serde(alias = "maxMilestonesToKeep")]
    max_milestones_to_keep: Option<u32>,
}

impl PruningMilestonesConfigBuilder {
    /// Sets whether pruning based on milestone indexes is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled.replace(enabled);
        self
    }

    /// Sets how many milestones to hold available in the storage.
    ///
    /// Note: You cannot set a value below [`MAX_MILESTONES_TO_KEEP_MINIMUM`].
    pub fn max_milestones_to_keep(mut self, max_milestones_to_keep: u32) -> Self {
        let max_milestones_to_keep = max_milestones_to_keep.max(MAX_MILESTONES_TO_KEEP_MINIMUM);
        self.max_milestones_to_keep.replace(max_milestones_to_keep);
        self
    }

    /// Finishes this builder into a [`PruningMilestonesConfig`].
    #[must_use]
    pub fn finish(self) -> PruningMilestonesConfig {
        PruningMilestonesConfig {
            enabled: self.enabled.unwrap_or(PRUNING_MILESTONES_ENABLED_DEFAULT),
            max_milestones_to_keep: self.max_milestones_to_keep.unwrap_or(MAX_MILESTONES_TO_KEEP_DEFAULT),
        }
    }
}

/// Builder for a [`PruningSizeConfig`].
#[derive(Default, Debug, Deserialize, PartialEq)]
#[must_use]
pub struct PruningSizeConfigBuilder {
    enabled: Option<bool>,
    #[serde(alias = "targetSize")]
    target_size: Option<String>,
    #[serde(alias = "thresholdPercentage")]
    threshold_percentage: Option<f32>,
    #[serde(alias = "cooldownTime")]
    cooldown_time: Option<String>,
}

impl PruningSizeConfigBuilder {
    /// Sets whether pruning based on storage size is enabled.
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

    /// Finishes this builder into a [`PruningSizeConfig`].
    #[must_use]
    pub fn finish(self) -> PruningSizeConfig {
        let target_size = self.target_size.unwrap_or_else(|| TARGET_SIZE_DEFAULT.to_string());
        let target_size = target_size
            .parse::<bytes::Bytes>()
            .expect("parse human-readable pruning target size")
            .size();

        let cooldown_time = self.cooldown_time.unwrap_or_else(|| COOLDOWN_TIME_DEFAULT.to_string());
        let cooldown_time =
            duration::parse(cooldown_time.as_ref()).expect("parse human-readable pruning cooldown time");

        PruningSizeConfig {
            enabled: self.enabled.unwrap_or(PRUNING_SIZE_ENABLED_DEFAULT),
            target_size,
            threshold_percentage: self.threshold_percentage.unwrap_or(THRESHOLD_PERCENTAGE_DEFAULT),
            cooldown_time,
        }
    }
}

/// Builder for a [`PruningReceiptsConfig`].
#[derive(Default, Debug, Deserialize, PartialEq)]
#[must_use]
pub struct PruningReceiptsConfigBuilder {
    enabled: Option<bool>,
}

impl PruningReceiptsConfigBuilder {
    /// Sets whether receipts will be pruned.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled.replace(enabled);
        self
    }

    /// Finishes this builder into a [`PruningReceiptsConfig`].
    #[must_use]
    pub fn finish(self) -> PruningReceiptsConfig {
        PruningReceiptsConfig {
            enabled: self.enabled.unwrap_or(PRUNING_RECEIPTS_ENABLED_DEFAULT),
        }
    }
}

/// The pruning configuration.
#[derive(Clone, Debug)]
pub struct PruningConfig {
    milestones: PruningMilestonesConfig,
    size: PruningSizeConfig,
    receipts: PruningReceiptsConfig,
}

impl PruningConfig {
    /// Returns a builder to create a [`PruningConfig`].
    pub fn build() -> PruningConfigBuilder {
        PruningConfigBuilder::new()
    }

    /// Returns the `[PruningMilestonesConfig`].
    #[inline(always)]
    pub fn milestones(&self) -> &PruningMilestonesConfig {
        &self.milestones
    }

    /// Returns the `[PruningSizeConfig`].
    #[inline(always)]
    pub fn size(&self) -> &PruningSizeConfig {
        &self.size
    }

    /// Returns the `[PruningReceiptsConfig`].
    #[inline(always)]
    pub fn receipts(&self) -> &PruningReceiptsConfig {
        &self.receipts
    }
}

/// The config associated with milestone index based pruning.
#[derive(Clone, Debug)]
pub struct PruningMilestonesConfig {
    enabled: bool,
    max_milestones_to_keep: u32,
}

impl PruningMilestonesConfig {
    /// Returns whether pruning based on milestone indexes is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the maximum number of milestones to hold available in the storage.
    pub fn max_milestones_to_keep(&self) -> u32 {
        self.max_milestones_to_keep
    }
}

/// The config associated with storage size based pruning.
#[derive(Clone, Debug)]
pub struct PruningSizeConfig {
    enabled: bool,
    target_size: usize,
    threshold_percentage: f32,
    cooldown_time: Duration,
}

impl PruningSizeConfig {
    /// Returns whether pruning based on a target storage size is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the target size of the database.
    pub fn target_size(&self) -> usize {
        self.target_size
    }

    /// Returns the percentage the database gets reduced if the target size is reached.
    pub fn threshold_percentage(&self) -> f32 {
        self.threshold_percentage
    }

    /// Returns the cooldown time between two pruning-by-database size events.
    pub fn cooldown_time(&self) -> Duration {
        self.cooldown_time
    }
}

/// The config associated with pruning receipts.
#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Clone, Debug)]
pub struct PruningReceiptsConfig {
    enabled: bool,
}

impl PruningReceiptsConfig {
    /// Returns whether pruning receipts is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct NodeConfig {
        pruning: PruningConfig,
    }

    #[derive(Default, Debug, Deserialize)]
    #[must_use]
    struct NodeConfigBuilder {
        pruning: Option<PruningConfigBuilder>,
    }

    impl NodeConfigBuilder {
        fn finish(self) -> NodeConfig {
            NodeConfig {
                pruning: self.pruning.unwrap().finish(),
            }
        }
    }

    fn create_config_from_json_str() -> PruningConfig {
        let config_json_str = r#"
        {
            "pruning": {
                "milestones": {
                    "enabled": false,
                    "maxMilestonesToKeep": 200
                },
                "size": {
                    "enabled": false,
                    "targetSize": "500MB",
                    "thresholdPercentage": 20.0,
                    "cooldownTime": "1m"
                },
                "receipts": {
                    "enabled": true
                }
            }
        }"#;

        let node_config = serde_json::from_str::<NodeConfigBuilder>(config_json_str)
            .expect("error deserializing json config str")
            .finish();

        node_config.pruning
    }

    fn create_config_from_toml_str() -> PruningConfig {
        let config_toml_str = r#"
        [pruning]
        [pruning.milestones]
        enabled                = false
        max_milestones_to_keep = 200
        [pruning.size]
        enabled                = false
        target_size            = "500MB"
        threshold_percentage   = 20.0
        cooldown_time          = "1m"
        [pruning.receipts]
        enabled                = true
        "#;

        let node_config_builder =
            toml::from_str::<NodeConfigBuilder>(config_toml_str).expect("error deserializing toml config str");

        println!("{:?}", node_config_builder);
        let node_config = node_config_builder.finish();

        node_config.pruning
    }

    #[test]
    fn deserialize_json_and_toml_repr_into_same_config() {
        let json_config = create_config_from_json_str();
        let toml_config = create_config_from_toml_str();

        assert!(!json_config.milestones().enabled());
        assert_eq!(json_config.milestones().max_milestones_to_keep(), 200);
        assert_eq!(json_config.size().target_size(), 500000000);
        assert_eq!(json_config.size().threshold_percentage(), 20.0);
        assert_eq!(json_config.size().cooldown_time(), Duration::from_secs(60));
        assert!(!json_config.size().enabled());
        assert!(json_config.receipts().enabled());

        assert!(!toml_config.milestones().enabled());
        assert_eq!(toml_config.milestones().max_milestones_to_keep(), 200);
        assert_eq!(toml_config.size().target_size(), 500000000);
        assert_eq!(toml_config.size().threshold_percentage(), 20.0);
        assert_eq!(toml_config.size().cooldown_time(), Duration::from_secs(60));
        assert!(!toml_config.size().enabled());
        assert!(toml_config.receipts().enabled());
    }
}
