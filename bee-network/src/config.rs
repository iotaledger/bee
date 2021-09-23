// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module containing types necessary for network (layer) configuration.

use crate::{
    identity::{Identity, LocalIdentity},
    util,
};

use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr, ToSocketAddrs},
};

/// Network configuration.
#[derive(Clone)]
pub struct GossipConfig {
    /// The bind address for the server accepting peers to exchange gossip.
    pub bind_addr: SocketAddr,
    /// The local identity of the node.
    pub local_id: LocalIdentity,
}

impl GossipConfig {
    /// Creates a new gossip config.
    pub fn new(bind_addr: SocketAddr, local_id: LocalIdentity) -> Self {
        Self { bind_addr, local_id }
    }
}

/// Serializable (and therefore persistable) network configuration data.
#[derive(Default, Serialize, Deserialize)]
#[serde(rename = "gossip")]
pub struct GossipConfigBuilder {
    #[serde(rename = "bindAddress")]
    bind_addr: Option<String>,
    #[serde(rename = "privateKey")]
    private_key: Option<String>,
}

impl GossipConfigBuilder {
    /// Sets the bind address for the gossip layer.
    pub fn bind_addr(&mut self, bind_addr: &String) {
        self.bind_addr.replace(bind_addr.clone());
    }
    /// Sets the private key for gossip layer authentication.
    pub fn private_key(&mut self, private_key: impl ToString) {
        self.private_key.replace(private_key.to_string());
    }
    /// Finishes the builder.
    pub fn finish(self) -> GossipConfig {
        GossipConfig {
            bind_addr: resolve_bind_addr(self.bind_addr.as_ref().unwrap()).expect("faulty bind address"),
            local_id: LocalIdentity::from_bs58_encoded_private_key(self.private_key.unwrap()),
        }
    }
}

fn resolve_bind_addr(bind_addr: &str) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    Ok(bind_addr.to_socket_addrs()?.next().ok_or(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "unresolvable bind address",
    ))?)
}

/// Stores connection and other information about a manual peer.
#[derive(Debug, Clone)]
pub struct ManualPeerConfig {
    /// The identity of the peer.
    pub identity: Identity,
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

    /// TODO: remove
    pub fn len(&self) -> usize {
        self.peer_configs.len()
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
    pub fn finish(self, local_id: &LocalIdentity) -> ManualPeerConfig {
        let ManualPeerConfigBuilder {
            public_key,
            address,
            alias,
        } = self;

        let alias = alias.unwrap();
        let public_key = util::from_public_key_str(public_key.unwrap());
        let is_dialer = public_key < local_id.public_key();

        let address: SocketAddr = address.unwrap().parse().expect("error parsing address");
        let identity = Identity::from_public_key(public_key);

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
    pub fn finish(self, local_id: &LocalIdentity) -> ManualPeeringConfig {
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
