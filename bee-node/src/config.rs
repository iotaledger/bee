// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Handles node configuration from a file.
//!
//! Note:
//! The idea is to have a common config file for all Bee node types to keep things as simple as possible.
//! Each node type grabs the information it needs from the config, and discards the rest.

use crate::{
    cli::ClArgs,
    local::Local,
    plugins::mqtt::config::{MqttConfig, MqttConfigBuilder},
    storage::NodeStorageBackend,
    util, BECH32_HRP_DEFAULT, NETWORK_NAME_DEFAULT,
};

#[cfg(feature = "dashboard")]
use crate::plugins::dashboard::config::{DashboardConfig, DashboardConfigBuilder};

use bee_autopeering::config::{AutopeeringConfig, AutopeeringConfigTomlBuilder};
use bee_common::logger::{LoggerConfig, LoggerConfigBuilder, LOGGER_STDOUT_NAME};
use bee_gossip::{NetworkConfig, NetworkConfigBuilder};
use bee_ledger::workers::{
    pruning::config::{PruningConfig, PruningConfigBuilder},
    snapshot::config::{SnapshotConfig, SnapshotConfigBuilder},
};
use bee_protocol::workers::config::{ProtocolConfig, ProtocolConfigBuilder};
use bee_rest_api::endpoints::config::{RestApiConfig, RestApiConfigBuilder};
use bee_tangle::config::{TangleConfig, TangleConfigBuilder};

use serde::Deserialize;

use std::{fs, path::Path};

/// Config file errors.
#[derive(Debug, thiserror::Error)]
pub enum NodeConfigError {
    #[error("Reading the config file failed. Caused by: {0}.")]
    FileRead(#[from] std::io::Error),
    #[error("Deserializing the config builder failed. Caused by: {0}.")]
    ConfigBuilderDeserialization(#[from] toml::de::Error),
}

/// Entails all data that can be stored in a Bee config file.
pub struct NodeConfig<S: NodeStorageBackend> {
    pub(crate) local: Local,
    pub(crate) network_spec: NetworkSpec,
    pub(crate) logger_config: LoggerConfig,
    pub(crate) gossip_config: NetworkConfig,
    pub(crate) autopeering_config: AutopeeringConfig,
    pub(crate) protocol_config: ProtocolConfig,
    pub(crate) rest_api_config: RestApiConfig,
    pub(crate) snapshot_config: SnapshotConfig,
    pub(crate) pruning_config: PruningConfig,
    pub(crate) storage_config: S::Config,
    pub(crate) tangle_config: TangleConfig,
    pub(crate) mqtt_config: MqttConfig,
    #[cfg(feature = "dashboard")]
    pub(crate) dashboard_config: DashboardConfig,
}

impl<S: NodeStorageBackend> NodeConfig<S> {
    /// Returns the logger config.
    pub fn logger_config(&self) -> &LoggerConfig {
        &self.logger_config
    }

    /// Returns whether this node should run as an autopeering entry node.
    pub fn run_as_entry_node(&self) -> bool {
        self.autopeering_config.enabled() && self.autopeering_config.run_as_entry_node()
    }
}

// NOTE: To make the config robust against refactoring we "serde-rename" all fields even if not strictly necessary.
/// A builder for a Bee config, that can be deserialized from a corresponding config file.
#[derive(Default, Deserialize)]
pub struct NodeConfigBuilder<S: NodeStorageBackend> {
    #[serde(rename = "identity")]
    pub(crate) identity: Option<String>,
    #[serde(rename = "alias")]
    pub(crate) alias: Option<String>,
    #[serde(rename = "bech32_hrp")]
    pub(crate) bech32_hrp: Option<String>,
    #[serde(rename = "network_id")]
    pub(crate) network_id: Option<String>,
    #[serde(rename = "logger")]
    pub(crate) logger_builder: Option<LoggerConfigBuilder>,
    #[serde(rename = "network")]
    pub(crate) gossip_builder: Option<NetworkConfigBuilder>,
    #[serde(rename = "autopeering")]
    pub(crate) autopeering_builder: Option<AutopeeringConfigTomlBuilder>,
    #[serde(rename = "protocol")]
    pub(crate) protocol_builder: Option<ProtocolConfigBuilder>,
    #[serde(rename = "rest_api")]
    pub(crate) rest_api_builder: Option<RestApiConfigBuilder>,
    #[serde(rename = "snapshot")]
    pub(crate) snapshot_builder: Option<SnapshotConfigBuilder>,
    #[serde(rename = "pruning")]
    pub(crate) pruning_builder: Option<PruningConfigBuilder>,
    #[serde(rename = "storage")]
    pub(crate) storage_builder: Option<S::ConfigBuilder>,
    #[serde(rename = "tangle")]
    pub(crate) tangle_builder: Option<TangleConfigBuilder>,
    #[serde(rename = "mqtt")]
    pub(crate) mqtt_builder: Option<MqttConfigBuilder>,
    #[cfg(feature = "dashboard")]
    #[serde(rename = "dashboard")]
    pub(crate) dashboard_builder: Option<DashboardConfigBuilder>,
}

impl<S: NodeStorageBackend> NodeConfigBuilder<S> {
    /// Creates a node config builder from a local config file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, NodeConfigError> {
        match fs::read_to_string(path) {
            Ok(toml) => toml::from_str::<Self>(&toml).map_err(NodeConfigError::ConfigBuilderDeserialization),
            Err(e) => Err(NodeConfigError::FileRead(e)),
        }
    }

    /// Applies commandline arguments to the builder.
    pub fn apply_args(mut self, args: &ClArgs) -> Self {
        // Overrride the log level.
        if let Some(log_level) = args.log_level() {
            if self.logger_builder.is_none() {
                self.logger_builder = Some(LoggerConfigBuilder::default());
            }
            // Panic: logger cannot be none at this point.
            self.logger_builder
                .as_mut()
                .unwrap()
                .level(LOGGER_STDOUT_NAME, log_level);
        }

        // Override the entry node mode.
        if args.run_as_entry_node() {
            if self.autopeering_builder.is_none() {
                self.autopeering_builder = Some(AutopeeringConfigTomlBuilder::default());
            }
            // Panic: autopeering cannot be none at this point.
            let autopeering_builder = self.autopeering_builder.as_mut().unwrap();
            autopeering_builder.enabled = true;
            autopeering_builder.run_as_entry_node = Some(true);
        }

        self
    }

    /// Returns the built node config.
    pub fn finish(self) -> NodeConfig<S> {
        // Create the local node entity.
        let local = if let Some(keypair) = self.identity {
            Local::from_keypair(keypair, self.alias).expect("error creating local from hex encoded keypair")
        } else {
            Local::new(self.alias)
        };

        // Create the necessary info about the network.
        let bech32_hrp = self.bech32_hrp.unwrap_or_else(|| BECH32_HRP_DEFAULT.to_owned());
        let network_name = self.network_id.unwrap_or_else(|| NETWORK_NAME_DEFAULT.to_string());
        let network_id = util::create_id_from_network_name(&network_name);

        let network_spec = NetworkSpec {
            name: network_name,
            id: network_id,
            hrp: bech32_hrp,
        };

        NodeConfig {
            local,
            network_spec,
            logger_config: self.logger_builder.unwrap_or_default().finish(),
            // TODO: Create specific error types for each config section, e.g.
            // Error::NetworkConfigError(bee_gossip::config::Error)
            gossip_config: self
                .gossip_builder
                .unwrap_or_default()
                .finish()
                .expect("faulty network configuration"),
            autopeering_config: self.autopeering_builder.unwrap_or_default().finish(),
            protocol_config: self.protocol_builder.unwrap_or_default().finish(),
            rest_api_config: self.rest_api_builder.unwrap_or_default().finish(),
            snapshot_config: self.snapshot_builder.unwrap_or_default().finish(),
            pruning_config: self.pruning_builder.unwrap_or_default().finish(),
            storage_config: self.storage_builder.unwrap_or_default().into(),
            tangle_config: self.tangle_builder.unwrap_or_default().finish(),
            mqtt_config: self.mqtt_builder.unwrap_or_default().finish(),
            #[cfg(feature = "dashboard")]
            dashboard_config: self.dashboard_builder.unwrap_or_default().finish(),
        }
    }
}

/// Represents an IOTA network specification. It consists of:
/// * a name, e.g. "chrysalis-mainnet";
/// * an id number (hash of the name);
/// * an "hrp"(human readable part) prefix for addresses used in this network;
#[derive(Clone, Debug)]
pub struct NetworkSpec {
    pub(crate) name: String,
    pub(crate) id: u64,
    pub(crate) hrp: String,
}

impl NetworkSpec {
    /// Returns the name of the network, e.g. "chrysalis-mainnet".
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the id of the network.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the bech32 encoded "hrp" (human readable part) of the network.
    pub fn hrp(&self) -> &str {
        &self.hrp
    }
}
