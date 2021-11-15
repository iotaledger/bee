// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::NetworkSpec, local::Local, plugins::mqtt::config::MqttConfig, storage::StorageBackend, util, NodeConfig,
    NodeConfigBuilder, BECH32_HRP_DEFAULT, NETWORK_NAME_DEFAULT,
};

#[cfg(feature = "dashboard")]
use crate::plugins::dashboard::config::DashboardConfig;

use bee_autopeering::config::{AutopeeringConfig, AutopeeringTomlConfig};
use bee_common::logger::{LoggerConfig, LoggerConfigBuilder, LOGGER_STDOUT_NAME};
use bee_gossip::{Keypair, NetworkConfig, NetworkConfigBuilder, PeerId, PublicKey};
use bee_ledger::workers::{
    pruning::config::{PruningConfig, PruningConfigBuilder},
    snapshot::config::{SnapshotConfig, SnapshotConfigBuilder},
};
use bee_protocol::workers::config::{ProtocolConfig, ProtocolConfigBuilder};
use bee_rest_api::endpoints::config::{RestApiConfig, RestApiConfigBuilder};
use bee_tangle::config::{TangleConfig, TangleConfigBuilder};

/// The config of a Bee full node.
pub struct FullNodeConfig<B: StorageBackend> {
    pub local: Local,
    pub network: NetworkSpec,
    pub logger: LoggerConfig,
    pub gossip: NetworkConfig,
    pub autopeering: AutopeeringConfig,
    pub protocol: ProtocolConfig,
    pub rest_api: RestApiConfig,
    pub snapshot: SnapshotConfig,
    pub pruning: PruningConfig,
    pub storage: B::Config,
    pub tangle: TangleConfig,
    pub mqtt: MqttConfig,
    #[cfg(feature = "dashboard")]
    pub dashboard: DashboardConfig,
}

impl<B: StorageBackend> FullNodeConfig<B> {
    /// Returns the local entity associated with the node.
    pub fn local(&self) -> &Local {
        &self.local
    }

    /// Returns the network specification.
    pub fn network_spec(&self) -> &NetworkSpec {
        &self.network
    }
}

impl<B: StorageBackend> Clone for FullNodeConfig<B> {
    fn clone(&self) -> Self {
        Self {
            local: self.local.clone(),
            network: self.network.clone(),
            logger: self.logger.clone(),
            gossip: self.gossip.clone(),
            autopeering: self.autopeering.clone(),
            protocol: self.protocol.clone(),
            rest_api: self.rest_api.clone(),
            snapshot: self.snapshot.clone(),
            pruning: self.pruning.clone(),
            storage: self.storage.clone(),
            tangle: self.tangle.clone(),
            mqtt: self.mqtt.clone(),
            #[cfg(feature = "dashboard")]
            dashboard: self.dashboard.clone(),
        }
    }
}

impl<S: StorageBackend> From<NodeConfig<S>> for FullNodeConfig<S> {
    fn from(node_cfg: NodeConfig<S>) -> Self {
        Self {
            local: node_cfg.local,
            network: node_cfg.network,
            logger: node_cfg.logger,
            gossip: node_cfg.gossip,
            autopeering: node_cfg.autopeering,
            protocol: node_cfg.protocol,
            rest_api: node_cfg.rest_api,
            snapshot: node_cfg.snapshot,
            pruning: node_cfg.pruning,
            storage: node_cfg.storage,
            tangle: node_cfg.tangle,
            mqtt: node_cfg.mqtt,
            #[cfg(feature = "dashboard")]
            dashboard: node_cfg.dashboard,
        }
    }
}
