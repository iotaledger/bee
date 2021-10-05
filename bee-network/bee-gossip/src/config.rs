// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Gossip (layer) configuration.

use serde::{Deserialize, Serialize};

use std::net::{SocketAddr, ToSocketAddrs};

/// Gossip configuration.
#[derive(Clone)]
pub struct GossipConfig {
    /// The bind address for the server accepting peers to exchange gossip.
    pub bind_addr: SocketAddr,
}

impl GossipConfig {
    /// Creates a new gossip config.
    pub fn new(bind_addr: SocketAddr) -> Self {
        Self { bind_addr }
    }
}

/// Serializable (and therefore persistable) gossip configuration data.
#[derive(Default, Serialize, Deserialize)]
#[serde(rename = "gossip")]
pub struct GossipConfigBuilder {
    #[serde(rename = "bindAddress")]
    bind_addr: Option<String>,
}

impl GossipConfigBuilder {
    /// Sets the bind address for the gossip layer.
    pub fn bind_addr(&mut self, bind_addr: &str) {
        self.bind_addr.replace(bind_addr.to_owned());
    }

    /// Finishes the builder.
    pub fn finish(self) -> GossipConfig {
        GossipConfig {
            bind_addr: resolve_bind_addr(self.bind_addr.as_ref().unwrap()).expect("faulty bind address"),
        }
    }
}

fn resolve_bind_addr(bind_addr: &str) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    Ok(bind_addr
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "unresolvable bind address"))?)
}
