// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bee_network::{Multiaddr, PeerId};

use std::str::FromStr;

const DEFAULT_BIND_ADDRESS: &str = "/ip4/127.0.0.1/tcp/1337";
const DEFAULT_MESSAGE: &str = "hello world";

pub struct ConfigBuilder {
    bind_address: Option<Multiaddr>,
    peers: Vec<String>,
    message: Option<String>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            bind_address: None,
            peers: vec![],
            message: None,
        }
    }

    pub fn with_bind_address(mut self, bind_address: String) -> Self {
        self.bind_address
            .replace(Multiaddr::from_str(&bind_address).expect("create Multiaddr instance"));
        self
    }

    pub fn with_peer_address(mut self, peer_address: String) -> Self {
        self.peers.push(peer_address);
        self
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message.replace(message);
        self
    }

    pub fn finish(self) -> Config {
        let peers = self
            .peers
            .iter()
            .map(|s| Multiaddr::from_str(s).expect("error parsing Multiaddr"))
            .collect();

        Config {
            bind_address: self
                .bind_address
                .unwrap_or_else(|| Multiaddr::from_str(DEFAULT_BIND_ADDRESS).unwrap()),
            peers,
            message: self.message.unwrap_or_else(|| DEFAULT_MESSAGE.into()),
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub bind_address: Multiaddr,
    pub peers: Vec<Multiaddr>,
    pub message: String,
}

impl Config {
    pub fn build() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}
