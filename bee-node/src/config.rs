// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "dashboard")]
use crate::plugins::dashboard::config::{DashboardConfig, DashboardConfigBuilder};

use crate::plugins::mqtt::config::{MqttConfig, MqttConfigBuilder};

use bee_common::logger::{LoggerConfig, LoggerConfigBuilder};
use bee_ledger::workers::{
    consensus::config::{LedgerConfig, LedgerConfigBuilder},
    pruning::config::{PruningConfig, PruningConfigBuilder},
    snapshot::config::{SnapshotConfig, SnapshotConfigBuilder},
};
use bee_network::{Keypair, NetworkConfig, NetworkConfigBuilder, PeerId, PublicKey};
use bee_protocol::workers::config::{ProtocolConfig, ProtocolConfigBuilder};
use bee_rest_api::endpoints::config::{RestApiConfig, RestApiConfigBuilder};
use bee_storage::backend::StorageBackend;
use bee_tangle::config::{TangleConfig, TangleConfigBuilder};

use crypto::hashes::{blake2b::Blake2b256, Digest};
use serde::Deserialize;
use thiserror::Error;

use std::{convert::TryInto, fs, path::Path};

const DEFAULT_ALIAS: &str = "bee";
const DEFAULT_BECH32_HRP: &str = "iota";
const DEFAULT_NETWORK_ID: &str = "c2-mainnet";

/// An error that may be generated when reading configuration data.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Reading the specified config file failed: {0}.")]
    ConfigFileReadFailure(#[from] std::io::Error),
    #[error("Deserializing the node config builder failed: {0}.")]
    NodeConfigBuilderCreationFailure(#[from] toml::de::Error),
}

/// A builder type for creating node configuration data.
#[derive(Default, Deserialize)]
pub struct NodeConfigBuilder<B: StorageBackend> {
    pub(crate) identity: Option<String>,
    pub(crate) alias: Option<String>,
    pub(crate) bech32_hrp: Option<String>,
    pub(crate) network_id: Option<String>,
    pub(crate) logger: Option<LoggerConfigBuilder>,
    pub(crate) network: Option<NetworkConfigBuilder>,
    pub(crate) protocol: Option<ProtocolConfigBuilder>,
    pub(crate) rest_api: Option<RestApiConfigBuilder>,
    pub(crate) snapshot: Option<SnapshotConfigBuilder>,
    pub(crate) pruning: Option<PruningConfigBuilder>,
    pub(crate) storage: Option<B::ConfigBuilder>,
    pub(crate) tangle: Option<TangleConfigBuilder>,
    pub(crate) ledger: Option<LedgerConfigBuilder>,
    pub(crate) mqtt: Option<MqttConfigBuilder>,
    #[cfg(feature = "dashboard")]
    pub(crate) dashboard: Option<DashboardConfigBuilder>,
}

impl<B: StorageBackend> NodeConfigBuilder<B> {
    /// Creates a node config builder from a local config file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        match fs::read_to_string(path) {
            Ok(toml) => toml::from_str::<Self>(&toml).map_err(Error::NodeConfigBuilderCreationFailure),
            Err(e) => Err(Error::ConfigFileReadFailure(e)),
        }
    }

    /// Finish using this node configuration builder, turning it into a node configuration.
    pub fn finish(self) -> NodeConfig<B> {
        let (identity, identity_string, new) = if let Some(identity_string) = self.identity {
            if identity_string.len() == 128 {
                let mut decoded = [0u8; 64];
                hex::decode_to_slice(&identity_string[..], &mut decoded).expect("error decoding identity");
                let identity = Keypair::decode(&mut decoded).expect("error decoding identity");
                (identity, identity_string, false)
            } else if identity_string.is_empty() {
                generate_random_identity()
            } else {
                panic!("invalid identity string length");
            }
        } else {
            generate_random_identity()
        };

        let node_id = PeerId::from_public_key(PublicKey::Ed25519(identity.public()));

        let network_id_string = self.network_id.unwrap_or_else(|| DEFAULT_NETWORK_ID.to_string());
        let network_id_numeric = u64::from_le_bytes(
            Blake2b256::digest(network_id_string.as_bytes())[0..8]
                .try_into()
                .unwrap(),
        );

        NodeConfig {
            identity: (identity, identity_string, new),
            node_id,
            alias: self.alias.unwrap_or_else(|| DEFAULT_ALIAS.to_owned()),
            bech32_hrp: self.bech32_hrp.unwrap_or_else(|| DEFAULT_BECH32_HRP.to_owned()),
            network_id: (network_id_string, network_id_numeric),
            logger: self.logger.unwrap_or_default().finish(),
            network: self.network.unwrap_or_default().finish(),
            protocol: self.protocol.unwrap_or_default().finish(),
            rest_api: self.rest_api.unwrap_or_default().finish(),
            snapshot: self.snapshot.unwrap_or_default().finish(),
            pruning: self.pruning.unwrap_or_default().finish(),
            storage: self.storage.unwrap_or_default().into(),
            tangle: self.tangle.unwrap_or_default().finish(),
            ledger: self.ledger.unwrap_or_default().finish(),
            mqtt: self.mqtt.unwrap_or_default().finish(),
            #[cfg(feature = "dashboard")]
            dashboard: self.dashboard.unwrap_or_default().finish(),
        }
    }
}

fn generate_random_identity() -> (Keypair, String, bool) {
    let identity = Keypair::generate();
    let encoded = identity.encode();
    let identity_string = hex::encode(encoded);
    (identity, identity_string, true)
}

/// Represents the configuration state of a node.
pub struct NodeConfig<B: StorageBackend> {
    /// The node's cryptographic identity.
    pub identity: (Keypair, String, bool),
    /// The node's peer ID.
    pub node_id: PeerId,
    /// The node's alias (unchecked string).
    pub alias: String,
    pub bech32_hrp: String,
    /// The node's network ID.
    pub network_id: (String, u64),
    /// The node's logger configuration.
    pub logger: LoggerConfig,
    /// The node's network configuration.
    pub network: NetworkConfig,
    /// The node's protocol configuration.
    pub protocol: ProtocolConfig,
    /// The node's ReST API configuration.
    pub rest_api: RestApiConfig,
    /// The node's snapshot configuration.
    pub snapshot: SnapshotConfig,
    /// The node's pruning configuration.
    pub pruning: PruningConfig,
    /// The node's storage backend configuration.
    pub storage: B::Config,
    /// The node's tangle configuration.
    pub tangle: TangleConfig,
    /// The node's ledger configuration.
    pub ledger: LedgerConfig,
    /// The node's MQTT configuration.
    pub mqtt: MqttConfig,
    /// The node's dashboard configuration.
    #[cfg(feature = "dashboard")]
    pub dashboard: DashboardConfig,
}

impl<B: StorageBackend> Clone for NodeConfig<B> {
    fn clone(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            node_id: self.node_id,
            alias: self.alias.clone(),
            bech32_hrp: self.bech32_hrp.clone(),
            network_id: self.network_id.clone(),
            logger: self.logger.clone(),
            network: self.network.clone(),
            protocol: self.protocol.clone(),
            rest_api: self.rest_api.clone(),
            snapshot: self.snapshot.clone(),
            pruning: self.pruning.clone(),
            storage: self.storage.clone(),
            tangle: self.tangle.clone(),
            ledger: self.ledger.clone(),
            mqtt: self.mqtt.clone(),
            #[cfg(feature = "dashboard")]
            dashboard: self.dashboard.clone(),
        }
    }
}
