// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::Multiaddr;
use serde::Deserialize;

use std::str::FromStr;

const DEFAULT_BIND_ADDRESS: &str = "/ip4/0.0.0.0/tcp/15600";

pub const DEFAULT_MSG_BUFFER_SIZE: usize = 10000;
pub const DEFAULT_KNOWN_PEER_LIMIT: usize = 6;
pub const DEFAULT_UNKNOWN_PEER_LIMIT: usize = 2;
pub const DEFAULT_RECONNECT_MILLIS: u64 = 60000;

/// Network configuration builder.
#[derive(Default, Deserialize)]
pub struct NetworkConfigBuilder {
    bind_address: Option<Multiaddr>,
    msg_buffer_size: Option<usize>,
    known_peer_limit: Option<usize>,
    unknown_peer_limit: Option<usize>,
    reconnect_millis: Option<u64>,
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

    pub fn msg_buffer_size(mut self, size: usize) -> Self {
        self.msg_buffer_size.replace(size);
        self
    }

    pub fn known_peer_limit(mut self, limit: usize) -> Self {
        self.known_peer_limit.replace(limit);
        self
    }

    pub fn unknown_peer_limit(mut self, limit: usize) -> Self {
        self.unknown_peer_limit.replace(limit);
        self
    }

    pub fn reconnect_millis(mut self, millis: u64) -> Self {
        self.reconnect_millis.replace(millis);
        self
    }

    /// Builds the network config.
    pub fn finish(self) -> NetworkConfig {
        NetworkConfig {
            bind_address: self
                .bind_address
                .unwrap_or(Multiaddr::from_str(DEFAULT_BIND_ADDRESS).unwrap()),
            msg_buffer_size: self.msg_buffer_size.unwrap_or(DEFAULT_MSG_BUFFER_SIZE),
            known_peer_limit: self.known_peer_limit.unwrap_or(DEFAULT_KNOWN_PEER_LIMIT),
            unknown_peer_limit: self.unknown_peer_limit.unwrap_or(DEFAULT_UNKNOWN_PEER_LIMIT),
            reconnect_millis: self.reconnect_millis.unwrap_or(DEFAULT_RECONNECT_MILLIS),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NetworkConfig {
    pub bind_address: Multiaddr,
    pub msg_buffer_size: usize,
    pub known_peer_limit: usize,
    pub unknown_peer_limit: usize,
    pub reconnect_millis: u64,
}

impl NetworkConfig {
    pub fn build() -> NetworkConfigBuilder {
        NetworkConfigBuilder::new()
    }
}
