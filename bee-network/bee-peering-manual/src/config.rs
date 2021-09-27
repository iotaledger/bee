// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module containing types necessary for network (layer) configuration.

use bee_identity::identity::{LocalId, PeerId};

use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
};

/// Stores connection and other information about a manual peer.
#[derive(Debug, Clone)]
pub struct ManualPeerConfig {
    /// The identity of the peer.
    pub identity: PeerId,
    /// The address of the peer.
    pub address: SocketAddr,
    /// A human friendly identifier of the peer.
    pub alias: String,
    /// Whether the peer is supposed to dial *us*.
    is_dialer: bool,
}

impl ManualPeerConfig {
    /// Whether the peer is supposed to be the initiator of a connection.
    pub fn is_dialer(&self) -> bool {
        self.is_dialer
    }
}

/// Manual peer configuration.
#[derive(Clone)]
pub struct ManualPeeringConfig {
    peer_configs: HashMap<IpAddr, ManualPeerConfig>,
}

impl ManualPeeringConfig {
    /// Returns a [`ManualPeerConfig`] associated with a particular [`IpAddr`].
    pub fn get(&self, ip_addr: &IpAddr) -> Option<&ManualPeerConfig> {
        self.peer_configs.get(ip_addr)
    }

    /// Adds a new static peer.
    pub fn add(&mut self, _config: ManualPeerConfig) -> bool {
        todo!("add manual peers")
    }

    /// Iterates over all manual peers.
    pub fn iter(&self) -> impl Iterator<Item = (&IpAddr, &ManualPeerConfig)> {
        self.peer_configs.iter()
    }
}

/// Serializable representation of a manual peer.
#[derive(Default, Serialize, Deserialize)]
pub struct ManualPeerConfigBuilder {
    #[serde(rename = "publicKey")]
    public_key: Option<String>,
    address: Option<String>,
    alias: Option<String>,
}

impl ManualPeerConfigBuilder {
    /// Finishes the builder.
    pub fn finish(self, local_id: &LocalId) -> ManualPeerConfig {
        let ManualPeerConfigBuilder {
            public_key,
            address,
            alias,
        } = self;

        let alias = alias.unwrap();
        let public_key = bee_identity::from_public_key_str(public_key.unwrap());
        let is_dialer = public_key < local_id.public_key();

        let address: SocketAddr = address.unwrap().parse().expect("error parsing address");
        let identity = PeerId::from_public_key(public_key);

        ManualPeerConfig {
            identity,
            address,
            alias,
            is_dialer,
        }
    }
}

/// Serializable representation of a list of manual peers.
#[derive(Default, Serialize, Deserialize)]
#[serde(rename = "manualPeering")]
pub struct ManualPeeringConfigBuilder {
    #[serde(rename = "knownPeers")]
    peer_config_builders: Vec<ManualPeerConfigBuilder>,
}

impl ManualPeeringConfigBuilder {
    /// Finishes the builder.
    pub fn finish(self, local_id: &LocalId) -> ManualPeeringConfig {
        let ManualPeeringConfigBuilder { peer_config_builders } = self;

        let mut peer_configs = HashMap::with_capacity(peer_config_builders.len());

        for peer_config_builder in peer_config_builders {
            let peer_config = peer_config_builder.finish(local_id);
            let ip = peer_config.address.ip();

            if peer_configs.contains_key(&ip) {
                unimplemented!("multiple instances with same ip address are intentionally not supported");
            }

            peer_configs.insert(ip, peer_config);
        }

        ManualPeeringConfig { peer_configs }
    }
}
