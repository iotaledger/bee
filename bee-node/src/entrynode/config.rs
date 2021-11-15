// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{NetworkSpec, NodeConfig, NodeConfigBuilder, NodeConfigError},
    local::Local,
    storage::StorageBackend,
    util, CliArgs, BECH32_HRP_DEFAULT, NETWORK_NAME_DEFAULT,
};

use bee_autopeering::{config::AutopeeringTomlConfig, AutopeeringConfig};
use bee_common::logger::{LoggerConfig, LoggerConfigBuilder, LOGGER_STDOUT_NAME};

use serde::Deserialize;

use std::{fs, path::Path};

/// The config of a Bee entry node.
#[derive(Clone)]
pub struct EntryNodeConfig {
    pub(crate) local: Local,
    pub(crate) network: NetworkSpec,
    pub(crate) logger: LoggerConfig,
    pub(crate) autopeering: AutopeeringConfig,
}

impl EntryNodeConfig {
    /// Returns the local entity.
    pub fn local(&self) -> &Local {
        &self.local
    }

    /// Returns the network specification the full node is trying to join.
    pub fn network_spec(&self) -> &NetworkSpec {
        &self.network
    }
}

impl<S: StorageBackend> From<NodeConfig<S>> for EntryNodeConfig {
    fn from(node_cfg: NodeConfig<S>) -> Self {
        Self {
            local: node_cfg.local,
            network: node_cfg.network,
            logger: node_cfg.logger,
            autopeering: node_cfg.autopeering,
        }
    }
}
