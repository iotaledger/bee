// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::Multiaddr;
use serde::Deserialize;

use std::str::FromStr;

const DEFAULT_BIND_ADDRESSES: &str = "/ip4/0.0.0.0/tcp/15600";

pub const DEFAULT_RECONNECT_INTERVAL_SECS: u64 = 30;

/// The network configuration.
#[derive(Clone, Debug)]
pub struct NetworkConfig {
    /// The address/es the networking layer tries binding to.
    pub bind_address: Multiaddr,
    /// The automatic reconnect interval in seconds for known peers.
    pub reconnect_interval_secs: u64,
    /// The configured entry nodes for peer discovery.
    pub entry_nodes: Vec<Multiaddr>,
}

impl NetworkConfig {
    /// Returns a network config builder.
    pub fn build() -> NetworkConfigBuilder {
        NetworkConfigBuilder::new()
    }
}

/// A network configuration builder.
/// Note: The fields of this struct have to correspond with the parameters in the config.
#[derive(Default, Deserialize)]
pub struct NetworkConfigBuilder {
    /// The address/es the networking layer tries binding to.
    bind_address: Option<Multiaddr>,
    /// The automatic reconnect interval in seconds for known peers.
    reconnect_interval_secs: Option<u64>,
    /// The configured entry nodes for peer discovery.
    dht: DhtConfigBuilder,
}

impl NetworkConfigBuilder {
    /// Creates a new default builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Specifies a bind address.
    pub fn bind_address(mut self, address: &str) -> Self {
        self.bind_address
            .replace(Multiaddr::from_str(address).unwrap_or_else(|e| panic!("Error parsing address: {:?}", e)));
        self
    }

    /// Specifies an interval in seconds for automatic reconnects.
    pub fn reconnect_interval_secs(mut self, secs: u64) -> Self {
        self.reconnect_interval_secs.replace(secs);
        self
    }

    /// Builds the network config.
    pub fn finish(self) -> NetworkConfig {
        NetworkConfig {
            bind_address: self
                .bind_address
                .unwrap_or_else(|| Multiaddr::from_str(DEFAULT_BIND_ADDRESSES).unwrap()),
            reconnect_interval_secs: self.reconnect_interval_secs.unwrap_or(DEFAULT_RECONNECT_INTERVAL_SECS),
            entry_nodes: self.dht.finish().entry_nodes,
        }
    }
}

#[derive(Clone)]
pub struct DhtConfig {
    pub entry_nodes: Vec<Multiaddr>,
}

impl DhtConfig {
    pub fn build() -> DhtConfigBuilder {
        DhtConfigBuilder::new()
    }
}

#[derive(Default, Deserialize)]
pub struct DhtConfigBuilder {
    pub entry_nodes: Option<Vec<EntryNode>>,
}

impl DhtConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entry_nodes(mut self, entry_nodes: Vec<EntryNode>) -> Self {
        self.entry_nodes.replace(entry_nodes);
        self
    }

    pub fn finish(self) -> DhtConfig {
        let entry_nodes = match self.entry_nodes {
            None => Vec::new(),
            Some(entry_nodes) => entry_nodes
                .into_iter()
                .map(|entry| Multiaddr::from_str(&entry.address[..]).expect("error parsing multiaddr."))
                .collect(),
        };

        DhtConfig { entry_nodes }
    }
}

#[derive(Deserialize)]
pub struct EntryNode {
    address: String,
}
