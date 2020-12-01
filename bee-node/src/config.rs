// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::logger::{LoggerConfig, LoggerConfigBuilder};
use bee_network::{NetworkConfig, NetworkConfigBuilder};
use bee_peering::{PeeringConfig, PeeringConfigBuilder};
use bee_protocol::config::{ProtocolConfig, ProtocolConfigBuilder};
use bee_snapshot::config::{SnapshotConfig, SnapshotConfigBuilder};
use bee_storage::storage::Backend;

use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use serde::Deserialize;
use thiserror::Error;

use std::{convert::TryInto, fs, path::Path};

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
    pub(crate) network_id: Option<String>,
    pub(crate) logger: Option<LoggerConfigBuilder>,
    pub(crate) network: Option<NetworkConfigBuilder>,
    pub(crate) peering: Option<PeeringConfigBuilder>,
    pub(crate) protocol: Option<ProtocolConfigBuilder>,
    pub(crate) snapshot: Option<SnapshotConfigBuilder>,
    pub(crate) database: Option<B::ConfigBuilder>,
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
            network_id,
            logger: self.logger.unwrap_or_default().finish(),
            network: self.network.unwrap_or_default().finish(),
            peering: self.peering.unwrap_or_default().finish(),
            protocol: self.protocol.unwrap_or_default().finish(),
            snapshot: self.snapshot.unwrap_or_default().finish(),
            database: self.database.unwrap_or_default().into(),
        }
    }
}

pub struct NodeConfig<B: Backend> {
    pub network_id: (String, u64),
    pub logger: LoggerConfig,
    pub network: NetworkConfig,
    pub peering: PeeringConfig,
    pub protocol: ProtocolConfig,
    pub snapshot: SnapshotConfig,
    pub database: B::Config,
}

impl<B: Backend> Clone for NodeConfig<B> {
    fn clone(&self) -> Self {
        Self {
            network_id: self.network_id.clone(),
            logger: self.logger.clone(),
            network: self.network.clone(),
            peering: self.peering.clone(),
            protocol: self.protocol.clone(),
            snapshot: self.snapshot.clone(),
            database: self.database.clone(),
        }
    }
}
