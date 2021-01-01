// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::{
    dashboard::config::{DashboardConfig, DashboardConfigBuilder},
    mqtt::config::{MqttConfig, MqttConfigBuilder},
};

use bee_common::logger::{LoggerConfig, LoggerConfigBuilder};
use bee_network::{NetworkConfig, NetworkConfigBuilder};
use bee_peering::{PeeringConfig, PeeringConfigBuilder};
use bee_protocol::config::{ProtocolConfig, ProtocolConfigBuilder};
use bee_rest_api::config::{RestApiConfig, RestApiConfigBuilder};
use bee_snapshot::config::{SnapshotConfig, SnapshotConfigBuilder};
use bee_storage::storage::Backend;

use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use serde::Deserialize;
use thiserror::Error;

use std::{convert::TryInto, fs, path::Path};

const DEFAULT_ALIAS: &str = "bee";
const DEFAULT_NETWORK_ID: &str = "alphanet1";

#[derive(Debug, Error)]
pub enum Error {
    #[error("Reading the specified config file failed: {0}.")]
    ConfigFileReadFailure(#[from] std::io::Error),
    #[error("Deserializing the node config builder failed: {0}.")]
    NodeConfigBuilderCreationFailure(#[from] toml::de::Error),
}

#[derive(Default, Deserialize)]
pub struct NodeConfigBuilder<B: Backend> {
    pub(crate) alias: Option<String>,
    pub(crate) network_id: Option<String>,
    pub(crate) logger: Option<LoggerConfigBuilder>,
    pub(crate) network: Option<NetworkConfigBuilder>,
    pub(crate) peering: Option<PeeringConfigBuilder>,
    pub(crate) protocol: Option<ProtocolConfigBuilder>,
    pub(crate) rest_api: Option<RestApiConfigBuilder>,
    pub(crate) snapshot: Option<SnapshotConfigBuilder>,
    pub(crate) storage: Option<B::ConfigBuilder>,
    pub(crate) mqtt: Option<MqttConfigBuilder>,
    pub(crate) dashboard: Option<DashboardConfigBuilder>,
}

impl<B: Backend> NodeConfigBuilder<B> {
    /// Creates a node config builder from a local config file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        match fs::read_to_string(path) {
            Ok(toml) => toml::from_str::<Self>(&toml).map_err(Error::NodeConfigBuilderCreationFailure),
            Err(e) => Err(Error::ConfigFileReadFailure(e)),
        }
    }

    pub fn finish(self) -> NodeConfig<B> {
        let mut hasher = VarBlake2b::new(32).unwrap();
        let mut network_id: (String, u64) = (self.network_id.unwrap_or_else(|| DEFAULT_NETWORK_ID.to_string()), 0);
        hasher.update(network_id.0.as_bytes());
        hasher.finalize_variable(|res| network_id.1 = u64::from_le_bytes(res[0..8].try_into().unwrap()));

        NodeConfig {
            alias: self.alias.unwrap_or_else(|| DEFAULT_ALIAS.to_owned()),
            network_id,
            logger: self.logger.unwrap_or_default().finish(),
            network: self.network.unwrap_or_default().finish(),
            peering: self.peering.unwrap_or_default().finish(),
            protocol: self.protocol.unwrap_or_default().finish(),
            rest_api: self.rest_api.unwrap_or_default().finish(),
            snapshot: self.snapshot.unwrap_or_default().finish(),
            storage: self.storage.unwrap_or_default().into(),
            mqtt: self.mqtt.unwrap_or_default().finish(),
            dashboard: self.dashboard.unwrap_or_default().finish(),
        }
    }
}

pub struct NodeConfig<B: Backend> {
    pub alias: String,
    pub network_id: (String, u64),
    pub logger: LoggerConfig,
    pub network: NetworkConfig,
    pub peering: PeeringConfig,
    pub protocol: ProtocolConfig,
    pub rest_api: RestApiConfig,
    pub snapshot: SnapshotConfig,
    pub storage: B::Config,
    pub mqtt: MqttConfig,
    pub dashboard: DashboardConfig,
}

impl<B: Backend> Clone for NodeConfig<B> {
    fn clone(&self) -> Self {
        Self {
            alias: self.alias.clone(),
            network_id: self.network_id.clone(),
            logger: self.logger.clone(),
            network: self.network.clone(),
            peering: self.peering.clone(),
            protocol: self.protocol.clone(),
            rest_api: self.rest_api.clone(),
            snapshot: self.snapshot.clone(),
            storage: self.storage.clone(),
            mqtt: self.mqtt.clone(),
            dashboard: self.dashboard.clone(),
        }
    }
}
