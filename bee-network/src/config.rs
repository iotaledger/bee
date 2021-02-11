// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::Multiaddr;
use serde::Deserialize;

use std::str::FromStr;

const DEFAULT_BIND_ADDRESS: &str = "/ip4/0.0.0.0/tcp/15600";

pub const DEFAULT_RECONNECT_INTERVAL_SECS: u64 = 30;

/// A network configuration builder.
#[derive(Default, Deserialize)]
pub struct NetworkConfigBuilder {
    /// The address/es the networking layer tries binding to.
    bind_address: Option<Multiaddr>,
    /// The automatic reconnect interval in seconds for known peers.
    reconnect_interval_secs: Option<u64>,
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
                .unwrap_or_else(|| Multiaddr::from_str(DEFAULT_BIND_ADDRESS).unwrap()),
            reconnect_interval_secs: self.reconnect_interval_secs.unwrap_or(DEFAULT_RECONNECT_INTERVAL_SECS),
        }
    }
}

/// The network configuration.
#[derive(Clone, Debug)]
pub struct NetworkConfig {
    /// The address/es the networking layer tries binding to.
    pub bind_address: Multiaddr,
    /// The automatic reconnect interval in seconds for known peers.
    pub reconnect_interval_secs: u64,
}

impl NetworkConfig {
    /// Returns a network config builder.
    pub fn build() -> NetworkConfigBuilder {
        NetworkConfigBuilder::new()
    }
}
