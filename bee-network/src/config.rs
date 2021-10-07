// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use crate::alias;

use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};
use serde::Deserialize;

use std::{borrow::Cow, collections::HashSet};

const DEFAULT_BIND_MULTIADDR: &str = "/ip4/0.0.0.0/tcp/15600";

pub const DEFAULT_RECONNECT_INTERVAL_SECS: u64 = 30;
const MIN_RECONNECT_INTERVAL_SECS: u64 = 1;

pub const DEFAULT_MAX_UNKNOWN_PEERS: usize = 4;
pub const DEFAULT_MAX_DISCOVERED_PEERS: usize = 4;

/// [`NetworkConfigBuilder`] errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The provided [`Multiaddr`] has too few protocols in it.
    #[error("Multiaddr is underspecified.")]
    MultiaddrUnderspecified,

    /// The provided [`Multiaddr`] has too many protocols in it.
    #[error("Multiaddr is overspecified.")]
    MultiaddrOverspecified,

    /// The provided [`Protocol`] is invalid.
    #[error("Invalid Multiaddr protocol at {}.", .0)]
    InvalidProtocol(usize),

    /// The provided address is invalid.
    #[error("Invalid address protocol.")]
    InvalidAddressProtocol,

    /// The provided port is invalid.
    #[error("Invalid port protocol.")]
    InvalidPortProtocol,

    /// The peer was already added.
    #[error("Static peer {} already added.", alias!(.0))]
    DuplicateStaticPeer(PeerId),

    /// The domain was unresolvable.
    #[error("Domain name '{}' couldn't be resolved to an IP address", .0)]
    UnresolvableDomain(String),

    /// Parsing of a [`Multiaddr`] failed.
    #[error("Parsing of '{}' to a Multiaddr failed.", 0)]
    ParsingFailed(String),

    /// The provided [`Multiaddr`] lacks the P2p [`Protocol`].
    #[error("Invalid P2p Multiaddr. Did you forget to add '.../p2p/12D3Koo...'?")]
    MissingP2pProtocol,
}

/// The network configuration.
#[derive(Clone)]
pub struct NetworkConfig {
    pub(crate) bind_multiaddr: Multiaddr,
    pub(crate) reconnect_interval_secs: u64,
    pub(crate) max_unknown_peers: usize,
    pub(crate) static_peers: HashSet<Peer>,
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

    /// Returns an in-memory config builder to construct a [`NetworkConfig`] iteratively.
    #[cfg(test)]
    pub fn build_in_memory() -> InMemoryNetworkConfigBuilder {
        InMemoryNetworkConfigBuilder::new()
    }

    /// Replaces the address, but keeps the port of the bind address.
    ///
    /// The argument `addr` must be either the `Ip4`, `Ip6`, or `Dns` variant of [`Protocol`].
    pub fn replace_addr(&mut self, mut addr: Protocol) -> Result<(), Error> {
        if !matches!(addr, Protocol::Ip4(_) | Protocol::Ip6(_) | Protocol::Dns(_)) {
            return Err(Error::InvalidAddressProtocol);
        }

        if let Protocol::Dns(dns) = addr {
            addr = resolve_dns_multiaddr(dns)?;
        }

        // Panic:
        // The builder ensures that the following unwraps are fine.
        let port = self.bind_multiaddr.pop().unwrap();

        let _ = self.bind_multiaddr.pop().unwrap();

        self.bind_multiaddr.push(addr);
        self.bind_multiaddr.push(port);

        Ok(())
    }

    /// Replaces the port of the bind address.
    ///
    /// The argument `port` must be the TCP variant of [`Protocol`].
    pub fn replace_port(&mut self, port: Protocol) -> Result<(), Error> {
        if !matches!(port, Protocol::Tcp(_)) {
            return Err(Error::InvalidPortProtocol);
        }

        self.bind_multiaddr.pop();
        self.bind_multiaddr.push(port);

        Ok(())
    }

    /// Adds a static peer.
    pub fn add_static_peer(
        &mut self,
        peer_id: PeerId,
        multiaddr: Multiaddr,
        alias: Option<String>,
    ) -> Result<(), Error> {
        if !self.static_peers.insert(Peer {
            peer_id,
            multiaddr,
            alias,
        }) {
            return Err(Error::DuplicateStaticPeer(peer_id));
        }

        Ok(())
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
    pub fn static_peers(&self) -> &HashSet<Peer> {
        &self.static_peers
    }
}

fn resolve_dns_multiaddr(dns: Cow<'_, str>) -> Result<Protocol, Error> {
    use std::net::{IpAddr, ToSocketAddrs};

    match dns
        .to_socket_addrs()
        .map_err(|_| Error::UnresolvableDomain(dns.to_string()))?
        .next()
        .ok_or_else(|| Error::UnresolvableDomain(dns.to_string()))?
        .ip()
    {
        IpAddr::V4(ip4) => return Ok(Protocol::Ip4(ip4)),
        IpAddr::V6(ip6) => return Ok(Protocol::Ip6(ip6)),
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            // Panic:
            // Unwrapping is fine, because we made sure that the default is parsable.
            bind_multiaddr: DEFAULT_BIND_MULTIADDR.parse().unwrap(),
            reconnect_interval_secs: DEFAULT_RECONNECT_INTERVAL_SECS,
            max_unknown_peers: DEFAULT_MAX_UNKNOWN_PEERS,
            static_peers: Default::default(),
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
    pub fn with_bind_multiaddr(mut self, mut multiaddr: Multiaddr) -> Result<Self, Error> {
        let mut valid = false;
        let mut is_dns = false;

        for (i, p) in multiaddr.iter().enumerate() {
            match i {
                0 => {
                    if !matches!(p, Protocol::Ip4(_) | Protocol::Ip6(_) | Protocol::Dns(_)) {
                        return Err(Error::InvalidProtocol(0));
                    }

                    if matches!(p, Protocol::Dns(_)) {
                        is_dns = true;
                    }
                }
                1 => {
                    if !matches!(p, Protocol::Tcp(_)) {
                        return Err(Error::InvalidProtocol(1));
                    }
                    valid = true;
                }
                _ => return Err(Error::MultiaddrOverspecified),
            }
        }
        if !valid {
            return Err(Error::MultiaddrUnderspecified);
        }

        if is_dns {
            let port = multiaddr.pop().unwrap();
            let port = if let Protocol::Tcp(port) = port {
                port
            } else {
                unreachable!("already checked");
            };
            // Panic:
            // We know at this point, that `multiaddr` is valid, so unwrapping is fine.
            let ip = if let Protocol::Dns(dns) = multiaddr.pop().unwrap() {
                let socket_dns = {
                    let mut socket_addr = String::with_capacity(16);
                    socket_addr.push_str(&dns);
                    socket_addr.push(':');
                    socket_addr.push_str(&port.to_string());
                    socket_addr
                };

                resolve_dns_multiaddr(socket_dns.into())?
            } else {
                unreachable!("already checked");
            };

            multiaddr.push(ip);
            multiaddr.push(Protocol::Tcp(port));
        }

        self.bind_multiaddr.replace(multiaddr);
        Ok(self)
    }

    /// Specifies the interval (in seconds) at which known peers are automatically reconnected if possible.
    ///
    /// The allowed minimum value for the `secs` argument is `1`.
    pub fn with_reconnect_interval_secs(mut self, secs: u64) -> Self {
        let secs = secs.max(MIN_RECONNECT_INTERVAL_SECS);
        self.reconnect_interval_secs.replace(secs);
        self
    }

    /// Specifies the maximum number of gossip connections with unknown peers.
    pub fn with_max_unknown_peers(mut self, n: usize) -> Self {
        self.max_unknown_peers.replace(n);
        self
    }

    /// Builds the network config.
    pub fn finish(self) -> Result<NetworkConfig, Error> {
        Ok(NetworkConfig {
            bind_multiaddr: self
                .bind_multiaddr
                // Panic:
                // We made sure that the default is parsable.
                .unwrap_or_else(|| DEFAULT_BIND_MULTIADDR.parse().unwrap()),
            reconnect_interval_secs: self.reconnect_interval_secs.unwrap_or(DEFAULT_RECONNECT_INTERVAL_SECS),
            max_unknown_peers: self.max_unknown_peers.unwrap_or(DEFAULT_MAX_UNKNOWN_PEERS),
            static_peers: self.peering.finish()?.peers,
        })
    }
}

/// An in-memory network config builder, that becomes useful as part of integration testing.
#[cfg(test)]
#[derive(Default)]
pub struct InMemoryNetworkConfigBuilder {
    bind_multiaddr: Option<Multiaddr>,
}

#[cfg(test)]
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
            max_unknown_peers: DEFAULT_MAX_UNKNOWN_PEERS,
            static_peers: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct PeeringConfig {
    pub peers: HashSet<Peer>,
}

#[derive(Clone)]
pub struct Peer {
    pub peer_id: PeerId,
    pub multiaddr: Multiaddr,
    pub alias: Option<String>,
}

impl Eq for Peer {}
impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.peer_id.eq(&other.peer_id)
    }
}
impl std::hash::Hash for Peer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.peer_id.hash(state)
    }
}

#[derive(Default, Deserialize)]
pub struct PeeringConfigBuilder {
    pub peers: Option<Vec<PeerBuilder>>,
}

impl PeeringConfigBuilder {
    pub fn finish(self) -> Result<PeeringConfig, Error> {
        let peers = match self.peers {
            None => Default::default(),
            Some(peer_builders) => {
                // NOTE: Switch back to combinators once `map_while` is stable.

                let mut peers = HashSet::with_capacity(peer_builders.len());

                for builder in peer_builders {
                    let (multiaddr, peer_id) = split_multiaddr(&builder.multiaddr)?;
                    if !peers.insert(Peer {
                        peer_id,
                        multiaddr,
                        alias: builder.alias,
                    }) {
                        return Err(Error::DuplicateStaticPeer(peer_id));
                    }
                }

                peers
            }
        };

        Ok(PeeringConfig { peers })
    }
}

fn split_multiaddr(multiaddr: &str) -> Result<(Multiaddr, PeerId), Error> {
    let mut multiaddr: Multiaddr = multiaddr
        .parse()
        .map_err(|_| Error::ParsingFailed(multiaddr.to_string()))?;

    if let Protocol::P2p(multihash) = multiaddr.pop().ok_or(Error::MultiaddrUnderspecified)? {
        Ok((
            multiaddr,
            PeerId::from_multihash(multihash).expect("Invalid peer Multiaddr: Make sure your peer's Id is complete."),
        ))
    } else {
        Err(Error::MissingP2pProtocol)
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
            .unwrap()
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
            .unwrap()
            .finish();
    }

    #[test]
    fn create_with_builder_and_valid_ip_bind_address() {
        let _config = NetworkConfig::build()
            .with_bind_multiaddr("/ip4/127.0.0.1/tcp/1337".parse().unwrap())
            .unwrap()
            .finish();
    }

    #[test]
    fn create_with_builder_and_valid_dns_bind_address() {
        let _config = NetworkConfig::build()
            .with_bind_multiaddr("/dns/localhost/tcp/1337".parse().unwrap())
            .unwrap()
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
}
