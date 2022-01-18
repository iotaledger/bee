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
    /// The local entity.
    pub local: Local,
    /// The specification of the network the node wants to participate in.
    pub network_spec: NetworkSpec,
    /// Logger.
    pub logger_config: LoggerConfig,
    /// Gossip layer.
    pub gossip_config: NetworkConfig,
    /// Autopeering.
    pub autopeering_config: AutopeeringConfig,
    /// Protocol layer.
    pub protocol_config: ProtocolConfig,
    /// Node REST API.
    pub rest_api_config: RestApiConfig,
    /// Snapshots.
    pub snapshot_config: SnapshotConfig,
    /// Pruning.
    pub pruning_config: PruningConfig,
    /// Storage layer.
    pub storage_config: S::Config,
    /// Tangle.
    pub tangle_config: TangleConfig,
    /// MQTT broker.
    pub mqtt_config: MqttConfig,
    /// Node dashboard.
    #[cfg(feature = "dashboard")]
    pub dashboard_config: DashboardConfig,
}

impl<S: NodeStorageBackend> FullNodeConfig<S> {
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
            local: local,
            network_spec: node_cfg.network_spec,
            logger_config: node_cfg.logger_config,
            gossip_config: node_cfg.gossip_config,
            autopeering_config: node_cfg.autopeering_config,
            protocol_config: node_cfg.protocol_config,
            rest_api_config: node_cfg.rest_api_config,
            snapshot_config: node_cfg.snapshot_config,
            pruning_config: node_cfg.pruning_config,
            storage_config: node_cfg.storage_config,
            tangle_config: node_cfg.tangle_config,
            mqtt_config: node_cfg.mqtt_config,
            #[cfg(feature = "dashboard")]
            dashboard_config: node_cfg.dashboard_config,
        }
    }
}

impl<S: NodeStorageBackend> Clone for FullNodeConfig<S> {
    fn clone(&self) -> Self {
        Self {
            local: self.local.clone(),
            network_spec: self.network_spec.clone(),
            logger_config: self.logger_config.clone(),
            gossip_config: self.gossip_config.clone(),
            autopeering_config: self.autopeering_config.clone(),
            protocol_config: self.protocol_config.clone(),
            rest_api_config: self.rest_api_config.clone(),
            snapshot_config: self.snapshot_config.clone(),
            pruning_config: self.pruning_config.clone(),
            storage_config: self.storage_config.clone(),
            tangle_config: self.tangle_config.clone(),
            mqtt_config: self.mqtt_config.clone(),
            #[cfg(feature = "dashboard")]
            dashboard_config: self.dashboard_config.clone(),
        }
    }
}
