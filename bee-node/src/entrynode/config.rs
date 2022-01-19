// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{NetworkSpec, NodeConfig},
    local::Local,
    storage::NodeStorageBackend,
};

use bee_autopeering::AutopeeringConfig;

use fern_logger::LoggerConfig;

/// The config of a Bee entry node.
#[derive(Clone)]
pub struct EntryNodeConfig {
    /// The local entity.
    pub local: Local,
    /// The specification of the network the node wants to participate in.
    pub network_spec: NetworkSpec,
    /// Logger.
    pub logger_config: LoggerConfig,
    /// Autopeering.
    pub autopeering_config: AutopeeringConfig,
}

impl EntryNodeConfig {
    /// Returns the local entity.
    pub fn local(&self) -> &Local {
        &self.local
    }

    /// Returns the network specification the full node is trying to join.
    pub fn network_spec(&self) -> &NetworkSpec {
        &self.network_spec
    }

    pub fn from<S>(local: Local, node_cfg: NodeConfig<S>) -> Self
    where
        S: NodeStorageBackend,
    {
        Self {
            local,
            network_spec: node_cfg.network_spec,
            logger_config: node_cfg.logger_config,
            autopeering_config: node_cfg.autopeering_config,
        }
    }
}
