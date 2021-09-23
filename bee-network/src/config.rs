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
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};

/// Represents a bind address.
#[derive(Clone, Debug)]
pub enum BindAddr {
    /// IP4 localhost.
    LocalhostV4 {
        /// The port to bind to at localhost.
        port: u16,
    },
    /// IP6 localhost.
    LocalhostV6 {
        /// The port to bind to at localhost.
        port: u16,
    },
    /// An arbitrary host.
    Host {
        /// The fully specified socket address to bind to.
        addr: SocketAddr,
    },
}

impl BindAddr {
    /// Converts this type into the corresponding [`SocketAddr`].
    pub fn into_socket_addr(self) -> SocketAddr {
        match self {
            BindAddr::LocalhostV4 { port } => SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
            BindAddr::LocalhostV6 { port } => SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), port),
            BindAddr::Host { addr } => addr,
        }
    }
}

/// Network configuration.
#[derive(Clone)]
pub struct NetworkConfig {
    /// The bind address for the server accepting peers to exchange gossip.
    pub bind_addr: SocketAddr,
    /// The local identity of the node.
    pub local_id: LocalIdentity,
}

impl NetworkConfig {
    /// Creates a new config.
    pub fn new(bind_addr: BindAddr, local_id: LocalIdentity) -> Self {
        Self {
            bind_addr: bind_addr.into_socket_addr(),
            local_id,
        }
    }
}

/// Serializable (and therefore persistable) network configuration data.
#[derive(Serialize, Deserialize)]
#[serde(rename = "network")]
pub struct NetworkConfigBuilder {
    #[serde(rename = "bindAddr")]
    bind_addr: Option<String>,
    #[serde(rename = "privateKey")]
    private_key: Option<String>,
}

impl NetworkConfigBuilder {
    /// Finishes the builder.
    pub fn finish(self) -> NetworkConfig {
        NetworkConfig {
            bind_addr: self.bind_addr.unwrap().parse().unwrap(),
            local_id: LocalIdentity::from_bs58_encoded_private_key(self.private_key.unwrap()),
        }
    }
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
#[derive(Serialize, Deserialize)]
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
