// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "standalone")]

use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};
use serde::Deserialize;

use std::str::FromStr;

const DEFAULT_BIND_MULTIADDR: &str = "/ip4/0.0.0.0/tcp/15600";

pub const DEFAULT_RECONNECT_INTERVAL_SECS: u64 = 30;
pub const DEFAULT_MAX_UNKOWN_PEERS: usize = 4;

/// The network configuration.
#[derive(Clone)]
pub struct NetworkConfig {
    /// Can represent a single or multiple ip addresses the network layer will try to bind to.
    pub bind_multiaddr: Multiaddr,
    /// The automatic reconnect interval in seconds for known peers.
    pub reconnect_interval_secs: u64,
    /// The maximum number of gossip connections with unknown peers.
    pub max_unknown_peers: usize,
    /// The static peers.
    pub peers: Vec<Peer>,
}

impl NetworkConfig {
    /// Creates a new `NetworkConfig`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `NetworkConfig` with a specific port.
    pub fn new_with_port(port: u16) -> Self {
        let mut this = Self::default();
        this.change_port(port);
        this
    }

    /// Returns a network config builder.
    pub fn build() -> NetworkConfigBuilder {
        NetworkConfigBuilder::new()
    }

    /// Changes the port.
    fn change_port(&mut self, port: u16) {
        self.bind_multiaddr.pop().unwrap();
        self.bind_multiaddr.push(Protocol::Tcp(port));
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            // `unwrap`ping the default never panics
            bind_multiaddr: Multiaddr::from_str(DEFAULT_BIND_MULTIADDR).unwrap(),
            reconnect_interval_secs: DEFAULT_RECONNECT_INTERVAL_SECS,
            max_unknown_peers: DEFAULT_MAX_UNKOWN_PEERS,
            peers: Vec::new(),
        }
    }
}

/// A network configuration builder.
#[derive(Default, Deserialize)]
pub struct NetworkConfigBuilder {
    #[serde(rename = "bind_multiaddress")]
    bind_multiaddr: Option<Multiaddr>,
    reconnect_interval_secs: Option<u64>,
    max_unknown_peers: Option<usize>,
    peering: PeeringConfigBuilder,
}

impl NetworkConfigBuilder {
    /// Creates a new default builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Specifies the bind addresses.
    pub fn bind_addresses(mut self, address: &str) -> Self {
        self.bind_multiaddr
            .replace(Multiaddr::from_str(address).unwrap_or_else(|e| panic!("Error parsing address: {:?}", e)));
        self
    }

    /// Specifies an interval in seconds for automatic reconnects.
    pub fn reconnect_interval_secs(mut self, secs: u64) -> Self {
        self.reconnect_interval_secs.replace(secs);
        self
    }

    /// The maximum number of gossip connections with unknown peers.
    pub fn max_unknown_peers(mut self, n: usize) -> Self {
        self.max_unknown_peers.replace(n);
        self
    }

    /// Builds the network config.
    pub fn finish(self) -> NetworkConfig {
        NetworkConfig {
            bind_multiaddr: self
                .bind_multiaddr
                .unwrap_or_else(|| Multiaddr::from_str(DEFAULT_BIND_MULTIADDR).unwrap()),

            reconnect_interval_secs: self.reconnect_interval_secs.unwrap_or(DEFAULT_RECONNECT_INTERVAL_SECS),

            max_unknown_peers: self.max_unknown_peers.unwrap_or(DEFAULT_MAX_UNKOWN_PEERS),

            peers: self.peering.finish().peers,
        }
    }
}

#[derive(Clone)]
pub struct PeeringConfig {
    pub peers: Vec<Peer>,
}

#[derive(Clone)]
pub struct Peer {
    pub peer_id: PeerId,
    pub multiaddr: Multiaddr,
    pub alias: Option<String>,
}

#[derive(Default, Deserialize)]
pub struct PeeringConfigBuilder {
    pub peers: Option<Vec<PeerBuilder>>,
}

impl PeeringConfigBuilder {
    pub fn finish(self) -> PeeringConfig {
        let peers = match self.peers {
             None => Vec::new(),
             Some(peer_builders) => peer_builders
                 .into_iter()
                 .map(|pb| {
                     // **Note**: this Multiaddr comes with the '../p2p/XYZ' suffix.
                     let mut p2p_addr = Multiaddr::from_str(&pb.multiaddr).expect("error parsing multiaddr.");

                     // NOTE: `unwrap`ing should be fine here since it comes from the config.
                     let (peer_id, multiaddr) = if let Protocol::P2p(multihash) = p2p_addr.pop().unwrap() {
                         let peer_id = PeerId::from_multihash(multihash).expect("Invalid Multiaddr.");
                         (peer_id, p2p_addr)
                     } else {
                         unreachable!(
                             "Invalid Peer descriptor. The multiaddress did not have a valid peer id as its last segment."
                         )
                     };

                     Peer {
                         peer_id,
                         multiaddr,
                         alias: pb.alias,
                     }
                 })
                 .collect(),
         };

        PeeringConfig { peers }
    }
}

#[derive(Deserialize)]
pub struct PeerBuilder {
    #[serde(rename = "multiaddress")]
    multiaddr: String,
    alias: Option<String>,
}
