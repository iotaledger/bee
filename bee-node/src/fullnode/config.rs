// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::NetworkSpec, local::Local, plugins::mqtt::config::MqttConfig, storage::NodeStorageBackend, NodeConfig,
};

#[cfg(feature = "dashboard")]
use crate::plugins::dashboard::config::DashboardConfig;

use bee_autopeering::config::AutopeeringConfig;
use bee_gossip::NetworkConfig;
use bee_ledger::workers::{pruning::config::PruningConfig, snapshot::config::SnapshotConfig};
use bee_protocol::workers::config::ProtocolConfig;
use bee_rest_api::endpoints::config::RestApiConfig;
use bee_tangle::config::TangleConfig;

use fern_logger::LoggerConfig;

/// The config of a Bee full node.
pub struct FullNodeConfig<S: NodeStorageBackend> {
    /// The node alias.
    pub alias: String,
    /// The local entity.
    pub local: Local,
    /// The specification of the network the node wants to participate in.
    pub network_spec: NetworkSpec,
    /// Logger.
    pub logger: LoggerConfig,
    /// Network layer.
    pub network: NetworkConfig,
    /// Autopeering.
    pub autopeering: AutopeeringConfig,
    /// Protocol layer.
    pub protocol: ProtocolConfig,
    /// Node REST API.
    pub rest_api: RestApiConfig,
    /// Snapshots.
    pub snapshot: SnapshotConfig,
    /// Pruning.
    pub pruning: PruningConfig,
    /// Storage layer.
    pub storage: S::Config,
    /// Tangle.
    pub tangle: TangleConfig,
    /// MQTT broker.
    pub mqtt: MqttConfig,
    /// Node dashboard.
    #[cfg(feature = "dashboard")]
    pub dashboard: DashboardConfig,
}

impl<S: NodeStorageBackend> FullNodeConfig<S> {
    /// Returns the alias of the node.
    pub fn alias(&self) -> &String {
        &self.alias
    }

    /// Returns the local entity associated with the node.
    pub fn local(&self) -> &Local {
        &self.local
    }

    /// Returns the network specification.
    pub fn network_spec(&self) -> &NetworkSpec {
        &self.network_spec
    }

    pub fn from(local: Local, node_cfg: NodeConfig<S>) -> Self {
        Self {
            alias: node_cfg.alias,
            local,
            network_spec: node_cfg.network_spec,
            logger: node_cfg.logger,
            network: node_cfg.network,
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

impl<S: NodeStorageBackend> Clone for FullNodeConfig<S> {
    fn clone(&self) -> Self {
        Self {
            alias: self.alias.clone(),
            local: self.local.clone(),
            network_spec: self.network_spec.clone(),
            logger: self.logger.clone(),
            network: self.network.clone(),
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
