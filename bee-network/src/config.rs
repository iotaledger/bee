// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};
use serde::Deserialize;

use std::{borrow::Cow, str::FromStr};

const DEFAULT_BIND_MULTIADDR: &str = "/ip4/0.0.0.0/tcp/15600";

pub const DEFAULT_RECONNECT_INTERVAL_SECS: u64 = 30;
const MIN_RECONNECT_INTERVAL_SECS: u64 = 1;

pub const DEFAULT_MAX_UNKOWN_PEERS: usize = 4;

/// The network configuration.
#[derive(Clone)]
pub struct NetworkConfig {
    pub(crate) bind_multiaddr: Multiaddr,
    pub(crate) reconnect_interval_secs: u64,
    pub(crate) max_unknown_peers: usize,
    pub(crate) static_peers: Vec<Peer>,
}

impl NetworkConfig {
    /// Creates a new [`NetworkConfig`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a [`NetworkConfigBuilder`] to construct a [`NetworkConfig`] iteratively.
    pub fn build() -> NetworkConfigBuilder {
        NetworkConfigBuilder::new()
    }

    /// Returns the configured bind address as a [`Multiaddr`].
    pub fn bind_multiaddr(&self) -> &Multiaddr {
        &self.bind_multiaddr
    }

    /// Returns the number of seconds at which reconnect attempts occur.
    pub fn reconnect_interval_secs(&self) -> u64 {
        self.reconnect_interval_secs
    }

    /// Returns the maximum number of unknown peers that are allowed to connect.
    pub fn max_unknown_peers(&self) -> usize {
        self.max_unknown_peers
    }

    /// Returns the statically configured peers.
    pub fn static_peers(&self) -> &Vec<Peer> {
        &self.static_peers
    }
}

// NOTE:
// This impl block is separated out because its mainly useful to simplify integration testing and examples.
// In the future we may consider to put this block behind a special feature flag so it doesn't get compiled
// into end-user binaries. Unfortunately we cannot expose it when attributed with `#[cfg(test)]`.
impl NetworkConfig {
    /// Returns a [`InMemoryNetworkConfigBuilder`] to construct a [`NetworkConfig`] iteratively.
    pub fn build_in_memory() -> InMemoryNetworkConfigBuilder {
        InMemoryNetworkConfigBuilder::new()
    }

    /// Replaces the address, but keeps the port of the bind address.
    ///
    /// The argument `addr` must be either the `Ip4`, `Ip6`, or `Dns` variant of [`Protocol`].
    pub fn replace_addr(&mut self, mut addr: Protocol) {
        if !matches!(addr, Protocol::Ip4(_) | Protocol::Ip6(_) | Protocol::Dns(_)) {
            panic!("Invalid addr");
        }

        if let Protocol::Dns(dns) = addr {
            addr = resolve_dns_multiaddr(dns);
        }

        let port = self.bind_multiaddr.pop().expect("multiaddr pop");
        let _ = self.bind_multiaddr.pop().expect("multiaddr pop");
        self.bind_multiaddr.push(addr);
        self.bind_multiaddr.push(port);
    }

    /// Replaces the port of the bind address.
    ///
    /// The argument `port` must be the TCP variant of [`Protocol`].
    pub fn replace_port(&mut self, port: Protocol) {
        if !matches!(port, Protocol::Tcp(_)) {
            panic!("Invalid port");
        }

        self.bind_multiaddr.pop();
        self.bind_multiaddr.push(port);
    }

    /// Adds a static peer.
    pub fn add_static_peer(&mut self, peer_id: PeerId, multiaddr: Multiaddr, alias: Option<String>) {
        self.static_peers.push(Peer {
            peer_id,
            multiaddr,
            alias,
        });
    }
}

fn resolve_dns_multiaddr(dns: Cow<'_, str>) -> Protocol {
    use std::net::{IpAddr, ToSocketAddrs};

    for socket_addr in dns.to_socket_addrs().expect("Invalid Multiaddr: Unresolvable") {
        match socket_addr.ip() {
            IpAddr::V4(ip4) => return Protocol::Ip4(ip4),
            IpAddr::V6(ip6) => return Protocol::Ip6(ip6),
        }
    }
    panic!("Invalid Multiaddr: Unresolvable");
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            // Panic:
            // Unwrapping is fine, because we made sure that the default is parseable.
            bind_multiaddr: DEFAULT_BIND_MULTIADDR.parse().unwrap(),
            reconnect_interval_secs: DEFAULT_RECONNECT_INTERVAL_SECS,
            max_unknown_peers: DEFAULT_MAX_UNKOWN_PEERS,
            static_peers: Vec::new(),
        }
    }
}

/// A network configuration builder.
#[derive(Default, Deserialize)]
pub struct NetworkConfigBuilder {
    #[serde(rename = "bind_address")]
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
    pub fn with_bind_multiaddr(mut self, mut multiaddr: Multiaddr) -> Self {
        let mut valid = false;
        let mut is_dns = false;

        for (i, p) in multiaddr.iter().enumerate() {
            match i {
                0 => {
                    if !matches!(p, Protocol::Ip4(_) | Protocol::Ip6(_) | Protocol::Dns(_)) {
                        panic!("Invalid Multiaddr: at index 0")
                    }

                    if matches!(p, Protocol::Dns(_)) {
                        is_dns = true;
                    }
                }
                1 => {
                    if !matches!(p, Protocol::Tcp(_)) {
                        panic!("Invalid Multiaddr: at index 1")
                    }
                    valid = true;
                }
                _ => panic!("Invalid Multiaddr: Too long"),
            }
        }
        if !valid {
            panic!("Invalid Multiaddr: Too short");
        }

        if is_dns {
            let port = multiaddr.pop().unwrap();
            let port = if let Protocol::Tcp(port) = port {
                port
            } else {
                unreachable!("already checked");
            };
            let ip = if let Protocol::Dns(dns) = multiaddr.pop().unwrap() {
                let socket_dns = {
                    let mut socket_addr = String::with_capacity(16);
                    socket_addr.push_str(&dns);
                    socket_addr.push_str(":");
                    socket_addr.push_str(&port.to_string());
                    socket_addr
                };

                resolve_dns_multiaddr(socket_dns.into())
            } else {
                unreachable!("already checked");
            };

            multiaddr.push(ip);
            multiaddr.push(Protocol::Tcp(port));
        }

        self.bind_multiaddr.replace(multiaddr);
        self
    }

    /// Specifies an interval in seconds for automatic reconnects.
    pub fn with_reconnect_interval_secs(mut self, secs: u64) -> Self {
        let secs = secs.max(MIN_RECONNECT_INTERVAL_SECS);
        self.reconnect_interval_secs.replace(secs);
        self
    }

    /// The maximum number of gossip connections with unknown peers.
    pub fn with_max_unknown_peers(mut self, n: usize) -> Self {
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
            static_peers: self.peering.finish().peers,
        }
    }
}

// Note:
// Ideally should be conditionally compiled, because there is hardly any use except for integration testing. See also
// note above.
/// An in-memory network config builder, that becomes useful as part of integration testing.
#[derive(Default)]
pub struct InMemoryNetworkConfigBuilder {
    bind_multiaddr: Option<Multiaddr>,
}

impl InMemoryNetworkConfigBuilder {
    /// Creates a new default builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Specifies the bind addresses.
    pub fn with_bind_multiaddr(mut self, multiaddr: Multiaddr) -> Self {
        for (i, p) in multiaddr.iter().enumerate() {
            match i {
                0 => {
                    if !matches!(p, Protocol::Memory(_)) {
                        panic!("Invalid Multiaddr")
                    }
                }
                _ => panic!("Invalid Multiaddr"),
            }
        }
        self.bind_multiaddr.replace(multiaddr);
        self
    }

    /// Builds the in-memory network config.
    pub fn finish(self) -> NetworkConfig {
        const DEFAULT_BIND_MULTIADDR_MEM: &str = "/memory/0";

        NetworkConfig {
            bind_multiaddr: self
                .bind_multiaddr
                .unwrap_or_else(|| DEFAULT_BIND_MULTIADDR_MEM.parse().unwrap()),
            reconnect_interval_secs: DEFAULT_RECONNECT_INTERVAL_SECS,
            max_unknown_peers: DEFAULT_MAX_UNKOWN_PEERS,
            static_peers: Vec::new(),
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
                .map(|builder| {
                    let (multiaddr, peer_id) = split_multiaddr(&builder.multiaddr);
                    Peer {
                        peer_id,
                        multiaddr,
                        alias: builder.alias,
                    }
                })
                .collect(),
        };

        PeeringConfig { peers }
    }
}

fn split_multiaddr(multiaddr: &str) -> (Multiaddr, PeerId) {
    let mut multiaddr: Multiaddr = multiaddr.parse().expect("error parsing Multiaddr");

    // Panic:
    // We want to panic if the configuration is faulty.
    if let Protocol::P2p(multihash) = multiaddr.pop().unwrap() {
        return (
            multiaddr,
            PeerId::from_multihash(multihash).expect("Invalid peer Multiaddr: Make sure your peer's Id is complete."),
        );
    } else {
        panic!("Invalid peer Multiaddr: Missing '.../p2p/12D3Koo...' suffix");
    }
}

#[derive(Deserialize)]
pub struct PeerBuilder {
    #[serde(rename = "address")]
    multiaddr: String,
    alias: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_default_network_config() {
        let config = NetworkConfig::default();

        assert_eq!(
            config.bind_multiaddr(),
            &DEFAULT_BIND_MULTIADDR.parse::<Multiaddr>().unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn create_with_builder_and_too_short_bind_address() {
        let _config = NetworkConfig::build()
            .with_bind_multiaddr("/ip4/127.0.0.1".parse().unwrap())
            .finish();
    }

    #[test]
    #[should_panic]
    fn create_with_builder_and_too_long_bind_address() {
        let _config = NetworkConfig::build()
            .with_bind_multiaddr(
                "/ip4/127.0.0.1/p2p/12D3KooWJWEKvSFbben74C7H4YtKjhPMTDxd7gP7zxWSUEeF27st"
                    .parse()
                    .unwrap(),
            )
            .finish();
    }

    #[test]
    fn create_with_builder_and_valid_ip_bind_address() {
        let _config = NetworkConfig::build()
            .with_bind_multiaddr("/ip4/127.0.0.1/tcp/1337".parse().unwrap())
            .finish();
    }

    #[test]
    fn create_with_builder_and_valid_dns_bind_address() {
        let _config = NetworkConfig::build()
            .with_bind_multiaddr("/dns/localhost/tcp/1337".parse().unwrap())
            .finish();
    }

    #[test]
    #[should_panic]
    fn create_with_mem_builder_and_non_mem_multiaddr() {
        let _config = NetworkConfig::build_in_memory()
            .with_bind_multiaddr("/ip4/127.0.0.1/tcp/1337".parse().unwrap())
            .finish();
    }

    #[test]
    fn create_with_mem_builder_and_valid_mem_multiaddr() {
        let _config = NetworkConfig::build_in_memory()
            .with_bind_multiaddr("/memory/1337".parse().unwrap())
            .finish();
    }

    #[test]
    fn create_with_peers() {
        // let _config = NetworkConfig::build().with_static_peers().
    }

    //     #[test]
    //     fn deserialize_config() {
    //         let s = r"[network]
    // # https://docs.libp2p.io/concepts/addressing/
    // bind_multiaddress       = "/ip4/0.0.0.0/tcp/15600"
    // reconnect_interval_secs = 30
    // max_unknown_peers       = 4

    // [network.peering]
    // #[[network.peering.peers]]
    // #multiaddress = ""
    // #alias        = ""
    // #[[network.peering.peers]]
    // #multiaddress = ""
    // #alias        = ""'
    //     }
}
