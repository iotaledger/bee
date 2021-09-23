// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing the node configuration.

use crate::cli::NodeCliArgs;

use bee_logger::{LoggerConfig, LoggerConfigBuilder, LOGGER_STDOUT_NAME};
use bee_network::config::{GossipConfig, GossipConfigBuilder, ManualPeeringConfig, ManualPeeringConfigBuilder};

use serde::Deserialize;
use thiserror::Error;

use std::{fs, path::Path};

/// Default path for the node configuration file.
pub const DEFAULT_NODE_CONFIG_PATH: &str = "./config.json";

/// Errors occurring while parsing the node configuration file.
#[derive(Debug, Error)]
pub enum Error {
    /// Reading the node configuration file failed.
    #[error("reading the node configuration file failed: {0}")]
    ConfigFileReadFailed(#[from] std::io::Error),
    /// Deserializing the node configuration file failed.
    #[error("deserializing the node configuration file failed: {0}")]
    ConfigFileDeserializationFailed(#[from] serde_json::error::Error),
}

/// Builder for the [`NodeConfig`].
#[derive(Default, Deserialize)]
pub struct NodeConfigBuilder {
    pub(crate) logger: Option<LoggerConfigBuilder>,
    pub(crate) gossip: Option<GossipConfigBuilder>,
    pub(crate) manual_peering: Option<ManualPeeringConfigBuilder>,
}

impl NodeConfigBuilder {
    /// Creates a [`NodeConfigBuilder`] from a config file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        match fs::read_to_string(path) {
            Ok(json) => serde_json::from_str::<Self>(&json).map_err(Error::ConfigFileDeserializationFailed),
            Err(e) => Err(Error::ConfigFileReadFailed(e)),
        }
    }

    /// Applies a [`NodeCliArgs`] to the [`NodeConfigBuilder`].
    pub fn with_cli_args(mut self, args: &NodeCliArgs) -> Self {
        if let Some(log_level) = args.log_level {
            if self.logger.is_none() {
                self.logger = Some(LoggerConfigBuilder::default());
            }
            // Unwrapping is fine because the logger is set to Some if it was None.
            self.logger.as_mut().unwrap().level(LOGGER_STDOUT_NAME, log_level);
        }

        if let Some(gossip_bind_addr) = args.gossip_bind_addr() {
            if self.gossip.is_none() {
                self.gossip = Some(GossipConfigBuilder::default());
            }
            // Unwrapping is fine because the gossip layer is set to Some if it was None.
            self.gossip.as_mut().unwrap().bind_addr(gossip_bind_addr);
        }

        if let Some(private_key) = args.private_key() {
            if self.gossip.is_none() {
                self.gossip = Some(GossipConfigBuilder::default());
            }
            // Unwrapping is fine because the gossip layer is set to Some if it was None.
            self.gossip.as_mut().unwrap().private_key(private_key);
        }

        self
    }

    /// Finished the [`NodeConfigBuilder`] into a [`NodeConfig`].
    pub fn finish(self) -> NodeConfig {
        let network = self.gossip.expect("missing network configuration").finish();
        let manual_peering = self.manual_peering.unwrap_or_default().finish(&network.local_id);

        NodeConfig {
            logger: self.logger.unwrap_or_default().finish(),
            network,
            manual_peering,
        }
    }
}

/// Node configuration.
pub struct NodeConfig {
    /// Logger configuration.
    pub logger: LoggerConfig,
    /// Network configuration.
    pub network: GossipConfig,
    /// Manual peering configuration.
    pub manual_peering: ManualPeeringConfig,
}

impl Clone for NodeConfig {
    fn clone(&self) -> Self {
        Self {
            logger: self.logger.clone(),
            network: self.network.clone(),
            manual_peering: self.manual_peering.clone(),
        }
    }
}
