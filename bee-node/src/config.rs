// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Handles node configuration from a file.
//!
//! Note:
//! The idea is to have a common config file for all Bee node types to keep things as simple as possible.
//! Each node type grabs the information it needs from the config, and discards the rest.

use crate::{
    cli::CliArgs,
    local::Local,
    plugins::mqtt::config::{MqttConfig, MqttConfigBuilder},
    util, BECH32_HRP_DEFAULT, NETWORK_NAME_DEFAULT,
};

#[cfg(feature = "dashboard")]
use crate::plugins::dashboard::config::{DashboardConfig, DashboardConfigBuilder};

use bee_autopeering::config::{AutopeeringConfig, AutopeeringTomlConfig};
use bee_common::logger::{LoggerConfig, LoggerConfigBuilder, LOGGER_STDOUT_NAME};
use bee_gossip::{Keypair, NetworkConfig, NetworkConfigBuilder, PeerId, PublicKey};
use bee_ledger::workers::{
    pruning::config::{PruningConfig, PruningConfigBuilder},
    snapshot::config::{SnapshotConfig, SnapshotConfigBuilder},
};
use bee_protocol::workers::config::{ProtocolConfig, ProtocolConfigBuilder};
use bee_rest_api::endpoints::config::{RestApiConfig, RestApiConfigBuilder};
use bee_storage::backend::StorageBackend;
use bee_tangle::config::{TangleConfig, TangleConfigBuilder};

use crypto::hashes::{blake2b::Blake2b256, Digest};
use serde::Deserialize;

use std::{fs, path::Path};

#[derive(Debug, thiserror::Error)]
pub enum NodeConfigError {
    #[error("Reading the config file failed. Caused by: {0}.")]
    FileRead(#[from] std::io::Error),
    #[error("Deserializing the config builder failed. Caused by: {0}.")]
    ConfigBuilderDeserialization(#[from] toml::de::Error),
}

pub struct NodeConfig<S: StorageBackend> {
    pub local: Local,
    pub network: NetworkSpec,
    pub logger: LoggerConfig,
    pub gossip: NetworkConfig,
    pub autopeering: AutopeeringConfig,
    pub protocol: ProtocolConfig,
    pub rest_api: RestApiConfig,
    pub snapshot: SnapshotConfig,
    pub pruning: PruningConfig,
    pub storage: S::Config,
    pub tangle: TangleConfig,
    pub mqtt: MqttConfig,
    #[cfg(feature = "dashboard")]
    pub dashboard: DashboardConfig,
}

impl<S: StorageBackend> NodeConfig<S> {
    /// Returns whether this node should run as an autopeering entry node.
    pub fn run_as_entry_node(&self) -> bool {
        self.autopeering.run_as_entry_node
    }
}

#[derive(Default, Deserialize)]
pub struct NodeConfigBuilder<S: StorageBackend> {
    pub(crate) identity: Option<String>,
    pub(crate) alias: Option<String>,
    pub(crate) bech32_hrp: Option<String>,
    pub(crate) network_id: Option<String>,
    pub(crate) logger: Option<LoggerConfigBuilder>,
    pub(crate) network: Option<NetworkConfigBuilder>,
    pub(crate) autopeering: Option<AutopeeringTomlConfig>,
    pub(crate) protocol: Option<ProtocolConfigBuilder>,
    pub(crate) rest_api: Option<RestApiConfigBuilder>,
    pub(crate) snapshot: Option<SnapshotConfigBuilder>,
    pub(crate) pruning: Option<PruningConfigBuilder>,
    pub(crate) storage: Option<S::ConfigBuilder>,
    pub(crate) tangle: Option<TangleConfigBuilder>,
    pub(crate) mqtt: Option<MqttConfigBuilder>,
    #[cfg(feature = "dashboard")]
    pub(crate) dashboard: Option<DashboardConfigBuilder>,
}

impl<S: StorageBackend> NodeConfigBuilder<S> {
    /// Creates a node config builder from a local config file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, NodeConfigError> {
        match fs::read_to_string(path) {
            Ok(toml) => toml::from_str::<Self>(&toml).map_err(NodeConfigError::ConfigBuilderDeserialization),
            Err(e) => Err(NodeConfigError::FileRead(e)),
        }
    }

    /// Applies CLI arguments to the builder.
    pub fn with_cli_args(mut self, args: CliArgs) -> Self {
        if let Some(log_level) = args.log_level().copied() {
            if self.logger.is_none() {
                self.logger = Some(LoggerConfigBuilder::default());
            }
            self.logger.as_mut().unwrap().level(LOGGER_STDOUT_NAME, log_level);
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
            network: network_spec,
            logger: self.logger.unwrap_or_default().finish(),
            // TODO: Create specific error types for each config section, e.g.
            // Error::NetworkConfigError(bee_gossip::config::Error)
            gossip: self
                .network
                .unwrap_or_default()
                .finish()
                .expect("faulty network configuration"),
            autopeering: self.autopeering.map_or(AutopeeringConfig::default(), |c| c.finish()),
            protocol: self.protocol.unwrap_or_default().finish(),
            rest_api: self.rest_api.unwrap_or_default().finish(),
            snapshot: self.snapshot.unwrap_or_default().finish(),
            pruning: self.pruning.unwrap_or_default().finish(),
            storage: self.storage.unwrap_or_default().into(),
            tangle: self.tangle.unwrap_or_default().finish(),
            mqtt: self.mqtt.unwrap_or_default().finish(),
            #[cfg(feature = "dashboard")]
            dashboard: self.dashboard.unwrap_or_default().finish(),
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
