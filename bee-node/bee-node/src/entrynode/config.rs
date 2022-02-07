// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{NetworkSpec, NodeConfig},
    local::Local,
    storage::NodeStorageBackend,
};

use bee_autopeering::AutopeeringConfig;
use bee_rest_api::endpoints::config::RestApiConfig;

use fern_logger::LoggerConfig;

/// The config of a Bee entry node.
#[derive(Clone)]
pub struct EntryNodeConfig {
    /// The local entity.
    pub local: Local,
    /// The specification of the network the node wants to participate in.
    pub network_spec: NetworkSpec,
    /// Logger.
    pub logger: LoggerConfig,
    /// Autopeering.
    pub autopeering: AutopeeringConfig,
    /// REST API.
    pub rest_api: RestApiConfig,
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
            logger: node_cfg.logger,
            autopeering: node_cfg.autopeering,
            rest_api: node_cfg.rest_api,
        }
    }
}
