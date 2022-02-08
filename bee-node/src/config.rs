// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Handles the proper configuration of a node from a single config file.
//!
//! ## Note
//! All node types use a common config file (e.g. config.json), and simply ignore
//! those parameters they don't actually require.

use crate::{
    cli::ClArgs,
    plugins::mqtt::config::{MqttConfig, MqttConfigBuilder},
    storage::NodeStorageBackend,
    util, BECH32_HRP_DEFAULT, NETWORK_NAME_DEFAULT,
};

#[cfg(feature = "dashboard")]
use crate::plugins::dashboard::config::{DashboardConfig, DashboardConfigBuilder};

use bee_autopeering::config::{AutopeeringConfig, AutopeeringConfigBuilder};
use bee_gossip::{NetworkConfig, NetworkConfigBuilder};
use bee_ledger::workers::{
    pruning::config::{PruningConfig, PruningConfigBuilder},
    snapshot::config::{SnapshotConfig, SnapshotConfigBuilder},
};
use bee_protocol::workers::config::{ProtocolConfig, ProtocolConfigBuilder};
use bee_rest_api::endpoints::config::{RestApiConfig, RestApiConfigBuilder};
use bee_tangle::config::{TangleConfig, TangleConfigBuilder};

use fern_logger::{LoggerConfig, LoggerConfigBuilder, LOGGER_STDOUT_NAME};
use serde::Deserialize;

use std::{fs, path::Path};

pub(crate) const ALIAS_DEFAULT: &str = "bee";

/// Config file errors.
#[derive(Debug, thiserror::Error)]
pub enum NodeConfigError {
    #[error("reading the config file failed: {0}")]
    FileRead(#[from] std::io::Error),
    #[error("unsupported configuration file type")]
    UnsupportedConfigType,
    #[error("deserializing the json config builder failed: {0}")]
    JsonConfigBuilderDeserialization(#[from] serde_json::Error),
    #[error("deserializing the toml config builder failed: {0}")]
    TomlConfigBuilderDeserialization(#[from] toml::de::Error),
}

/// Entails all data that can be stored in a Bee config file.
pub struct NodeConfig<S: NodeStorageBackend> {
    pub(crate) alias: String,
    pub(crate) network_spec: NetworkSpec,
    pub(crate) logger: LoggerConfig,
    pub(crate) network: NetworkConfig,
    pub(crate) autopeering: AutopeeringConfig,
    pub(crate) protocol: ProtocolConfig,
    pub(crate) rest_api: RestApiConfig,
    pub(crate) snapshot: SnapshotConfig,
    pub(crate) pruning: PruningConfig,
    pub(crate) storage: S::Config,
    pub(crate) tangle: TangleConfig,
    pub(crate) mqtt: MqttConfig,
    #[cfg(feature = "dashboard")]
    pub(crate) dashboard: DashboardConfig,
}

impl<S: NodeStorageBackend> NodeConfig<S> {
    /// Returns the alias.
    pub fn alias(&self) -> &String {
        &self.alias
    }

    /// Returns the logger config.
    pub fn logger(&self) -> &LoggerConfig {
        &self.logger
    }

    /// Returns whether this node should run as an autopeering entry node.
    pub fn run_as_entry_node(&self) -> bool {
        self.autopeering.enabled() && self.autopeering.run_as_entry_node()
    }
}

// NOTE: To make the config robust against refactoring we "serde-rename" all fields even if not strictly necessary.
/// A builder for a Bee config, that can be deserialized from a corresponding config file.
#[derive(Default, Deserialize)]
#[must_use]
pub struct NodeConfigBuilder<S: NodeStorageBackend> {
    // We don't store the identity in the config file anymore.
    // This is here for legacy reasons to warn the user of that fact.
    #[deprecated(since = "0.3.0")]
    #[serde(alias = "identity")]
    _identity: Option<String>,
    pub(crate) alias: Option<String>,
    #[serde(alias = "bech32Hrp")]
    pub(crate) bech32_hrp: Option<String>,
    #[serde(alias = "networkId")]
    pub(crate) network_id: Option<String>,
    pub(crate) logger: Option<LoggerConfigBuilder>,
    pub(crate) network: Option<NetworkConfigBuilder>,
    pub(crate) autopeering: Option<AutopeeringConfigBuilder>,
    pub(crate) protocol: Option<ProtocolConfigBuilder>,
    #[serde(alias = "restApi")]
    pub(crate) rest_api: Option<RestApiConfigBuilder>,
    pub(crate) snapshot: Option<SnapshotConfigBuilder>,
    pub(crate) pruning: Option<PruningConfigBuilder>,
    pub(crate) storage: Option<S::ConfigBuilder>,
    pub(crate) tangle: Option<TangleConfigBuilder>,
    pub(crate) mqtt: Option<MqttConfigBuilder>,
    #[cfg(feature = "dashboard")]
    pub(crate) dashboard: Option<DashboardConfigBuilder>,
}

impl<S: NodeStorageBackend> NodeConfigBuilder<S> {
    /// Creates a node config builder from a local config file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, NodeConfigError> {
        match fs::read_to_string(&path) {
            Ok(string) => match path.as_ref().extension().and_then(|e| e.to_str()) {
                Some("json") => {
                    serde_json::from_str::<Self>(&string).map_err(NodeConfigError::JsonConfigBuilderDeserialization)
                }
                Some("toml") => {
                    toml::from_str::<Self>(&string).map_err(NodeConfigError::TomlConfigBuilderDeserialization)
                }
                _ => Err(NodeConfigError::UnsupportedConfigType),
            },
            Err(e) => Err(NodeConfigError::FileRead(e)),
        }
    }

    /// Applies commandline arguments to the builder.
    pub fn apply_args(mut self, args: &ClArgs) -> Self {
        // Override the log level.
        if let Some(log_level) = args.log_level() {
            // TODO: use 'option_get_or_insert_default' once stable (see issue #82901)
            let logger = self.logger.get_or_insert(LoggerConfigBuilder::default());

            logger.level(LOGGER_STDOUT_NAME, log_level);
        }

        // Override the entry node mode.
        if args.run_as_entry_node() {
            // TODO: use 'option_get_or_insert_default' once stable (see issue #82901)
            let autopeering = self.autopeering.get_or_insert(AutopeeringConfigBuilder::default());

            autopeering.enabled = true;
            autopeering.run_as_entry_node = Some(true);
        }

        self
    }

    /// Returns the built node config.
    pub fn finish(self) -> (Option<String>, NodeConfig<S>) {
        // Create the necessary info about the network.
        let bech32_hrp = self.bech32_hrp.unwrap_or_else(|| BECH32_HRP_DEFAULT.to_owned());
        let network_name = self.network_id.unwrap_or_else(|| NETWORK_NAME_DEFAULT.to_string());
        let network_id = util::create_id_from_network_name(&network_name);

        let network_spec = NetworkSpec {
            name: network_name,
            id: network_id,
            hrp: bech32_hrp,
        };

        (
            #[allow(deprecated)]
            self._identity,
            NodeConfig {
                alias: self.alias.unwrap_or_else(|| ALIAS_DEFAULT.to_owned()),
                network_spec,
                logger: self.logger.unwrap_or_default().finish(),
                network: self
                    .network
                    .unwrap_or_default()
                    .finish()
                    .expect("faulty network configuration"),
                autopeering: self.autopeering.unwrap_or_default().finish(),
                protocol: self.protocol.unwrap_or_default().finish(),
                rest_api: self.rest_api.unwrap_or_default().finish(),
                snapshot: self.snapshot.unwrap_or_default().finish(),
                pruning: self.pruning.unwrap_or_default().finish(),
                storage: self.storage.unwrap_or_default().into(),
                tangle: self.tangle.unwrap_or_default().finish(),
                mqtt: self.mqtt.unwrap_or_default().finish(),
                #[cfg(feature = "dashboard")]
                dashboard: self.dashboard.unwrap_or_default().finish(),
            },
        )
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

#[cfg(test)]
mod test {

    use super::*;

    #[cfg(feature = "rocksdb")]
    use bee_storage_rocksdb::storage::Storage;
    #[cfg(all(feature = "sled", not(feature = "rocksdb")))]
    use bee_storage_sled::storage::Storage;

    #[test]
    fn config_files_conformity() -> Result<(), NodeConfigError> {
        let _ = NodeConfigBuilder::<Storage>::from_file(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/config.stardust-testnet.json"
        ))?;
        let _ = NodeConfigBuilder::<Storage>::from_file(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/config.stardust-testnet.toml"
        ))?;
        Ok(())
    }
}
