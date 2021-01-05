// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::Multiaddr;
use serde::Deserialize;

use std::str::FromStr;

const DEFAULT_BIND_ADDRESS: &str = "/ip4/0.0.0.0/tcp/15600";

pub const DEFAULT_RECONNECT_INTERVAL_SECS: u64 = 30;
pub const DEFAULT_CONNECTION_TIMEOUT_SECS: u64 = 10;

/// Network configuration builder.
#[derive(Default, Deserialize)]
pub struct NetworkConfigBuilder {
    bind_address: Option<Multiaddr>,
    reconnect_interval_secs: Option<u64>,
}

impl NetworkConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_address(mut self, address: &str) -> Self {
        self.bind_address
            .replace(Multiaddr::from_str(address).unwrap_or_else(|e| panic!("Error parsing address: {:?}", e)));
        self
    }

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

#[derive(Clone, Debug)]
pub struct NetworkConfig {
    pub bind_address: Multiaddr,
    pub reconnect_interval_secs: u64,
}

impl NetworkConfig {
    pub fn build() -> NetworkConfigBuilder {
        NetworkConfigBuilder::new()
    }
}
