// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use crate::peer::peer_id::PeerId;

use libp2p::{multiaddr::Protocol, Multiaddr};
use serde::Deserialize;

use std::{borrow::Cow, collections::HashSet, hash, time::Duration};

const BIND_ADDR_DEFAULT: &str = "/ip4/0.0.0.0/tcp/15600";

pub const RECONNECT_INTERVAL_DEFAULT: Duration = Duration::from_secs(30);
const RECONNECT_INTERVAL_MIN: Duration = Duration::from_secs(1);

pub const MAX_UNKNOWN_PEERS_DEFAULT: u16 = 4;
pub const MAX_DISCOVERED_PEERS_DEFAULT: u16 = 4;

/// [`GossipLayerConfigBuilder`] errors.
#[derive(Debug, thiserror::Error)]
pub enum GossipLayerConfigError {
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
    #[error("Manual peer {0} already added.")]
    DuplicateManualPeer(PeerId),

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
pub struct GossipLayerConfig {
    pub(crate) bind_addr: Multiaddr,
    pub(crate) reconnect_interval: Duration,
    pub(crate) max_unknown_peers: u16,
    pub(crate) max_discovered_peers: u16,
    pub(crate) manual_peers: Vec<PeerConfig>,
}

impl GossipLayerConfig {
    /// Creates a new [`NetworkConfig`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a [`GossipLayerConfigBuilder`] to construct a [`NetworkConfig`] iteratively.
    pub fn build() -> GossipLayerConfigBuilder {
        GossipLayerConfigBuilder::new()
    }

    /// Returns an in-memory config builder to construct a [`NetworkConfig`] iteratively.
    #[cfg(test)]
    pub fn build_in_memory() -> InMemoryNetworkConfigBuilder {
        InMemoryNetworkConfigBuilder::new()
    }

    /// Replaces the address, but keeps the port of the bind address.
    ///
    /// The argument `addr` must be either the `Ip4`, `Ip6`, or `Dns` variant of [`Protocol`].
    pub fn replace_addr(&mut self, mut addr: Protocol) -> Result<(), GossipLayerConfigError> {
        if !matches!(addr, Protocol::Ip4(_) | Protocol::Ip6(_) | Protocol::Dns(_)) {
            return Err(GossipLayerConfigError::InvalidAddressProtocol);
        }

        if let Protocol::Dns(dns) = addr {
            addr = resolve_dns_multiaddr(dns)?;
        }

        // Panic:
        // The builder ensures that the following unwraps are fine.
        let port = self.bind_addr.pop().unwrap();

        let _ = self.bind_addr.pop().unwrap();

        self.bind_addr.push(addr);
        self.bind_addr.push(port);

        Ok(())
    }

    /// Replaces the port of the bind address.
    ///
    /// The argument `port` must be the TCP variant of [`Protocol`].
    pub fn replace_port(&mut self, port: Protocol) -> Result<(), GossipLayerConfigError> {
        if !matches!(port, Protocol::Tcp(_)) {
            return Err(GossipLayerConfigError::InvalidPortProtocol);
        }

        self.bind_addr.pop();
        self.bind_addr.push(port);

        Ok(())
    }

    /// Adds a manual peer.
    pub fn add_manual_peer(
        &mut self,
        peer_id: PeerId,
        peer_addr: Multiaddr,
        peer_alias: Option<String>,
    ) -> Result<(), GossipLayerConfigError> {
        let manual_peer = PeerConfig {
            peer_id,
            peer_addr,
            peer_alias,
        };

        if self.manual_peers.contains(&manual_peer) {
            Err(GossipLayerConfigError::DuplicateManualPeer(peer_id))
        } else {
            self.manual_peers.push(manual_peer);
            Ok(())
        }
    }

    /// Returns the configured bind address as a [`Multiaddr`].
    pub fn bind_multiaddr(&self) -> &Multiaddr {
        &self.bind_addr
    }

    /// Returns the number of seconds at which reconnect attempts occur.
    pub fn reconnect_interval(&self) -> Duration {
        self.reconnect_interval
    }

    /// Returns the maximum number of unknown peers that are allowed to connect.
    pub fn max_unknown_peers(&self) -> u16 {
        self.max_unknown_peers
    }

    /// Returns the maximum number of discovered peers that are allowed to connect.
    pub fn max_discovered_peers(&self) -> u16 {
        self.max_discovered_peers
    }

    /// Returns the manually configured peers.
    pub fn manual_peers(&self) -> &Vec<PeerConfig> {
        &self.manual_peers
    }
}

fn resolve_dns_multiaddr(dns: Cow<'_, str>) -> Result<Protocol, GossipLayerConfigError> {
    use std::net::{IpAddr, ToSocketAddrs};

    match dns
        .to_socket_addrs()
        .map_err(|_| GossipLayerConfigError::UnresolvableDomain(dns.to_string()))?
        .next()
        .ok_or_else(|| GossipLayerConfigError::UnresolvableDomain(dns.to_string()))?
        .ip()
    {
        IpAddr::V4(ip4) => return Ok(Protocol::Ip4(ip4)),
        IpAddr::V6(ip6) => return Ok(Protocol::Ip6(ip6)),
    }
}

impl Default for GossipLayerConfig {
    fn default() -> Self {
        Self {
            // Panic:
            // Unwrapping is fine, because we made sure that the default is parsable.
            bind_addr: BIND_ADDR_DEFAULT.parse().unwrap(),
            reconnect_interval: RECONNECT_INTERVAL_DEFAULT,
            max_unknown_peers: MAX_UNKNOWN_PEERS_DEFAULT,
            max_discovered_peers: MAX_DISCOVERED_PEERS_DEFAULT,
            manual_peers: Default::default(),
        }
    }
}

/// A network configuration builder.
#[derive(Default, Deserialize)]
#[must_use]
pub struct GossipLayerConfigBuilder {
    #[serde(alias = "bindAddress", alias = "bind_address")]
    bind_multiaddr: Option<Multiaddr>,
    #[serde(alias = "reconnectIntervalSecs")]
    reconnect_interval_secs: Option<u64>,
    #[serde(alias = "maxUnknownPeers")]
    max_unknown_peers: Option<u16>,
    #[serde(alias = "maxDiscoveredPeers")]
    max_discovered_peers: Option<u16>,
    peering: ManualPeeringConfigBuilder,
}

impl GossipLayerConfigBuilder {
    /// Creates a new default builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Specifies the bind addresses.
    pub fn with_bind_multiaddr(mut self, mut multiaddr: Multiaddr) -> Result<Self, GossipLayerConfigError> {
        let mut valid = false;
        let mut is_dns = false;

        for (i, p) in multiaddr.iter().enumerate() {
            match i {
                0 => {
                    if !matches!(p, Protocol::Ip4(_) | Protocol::Ip6(_) | Protocol::Dns(_)) {
                        return Err(GossipLayerConfigError::InvalidProtocol(0));
                    }

                    if matches!(p, Protocol::Dns(_)) {
                        is_dns = true;
                    }
                }
                1 => {
                    if !matches!(p, Protocol::Tcp(_)) {
                        return Err(GossipLayerConfigError::InvalidProtocol(1));
                    }
                    valid = true;
                }
                _ => return Err(GossipLayerConfigError::MultiaddrOverspecified),
            }
        }
        if !valid {
            return Err(GossipLayerConfigError::MultiaddrUnderspecified);
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
        let secs = secs.max(RECONNECT_INTERVAL_MIN.as_secs());
        self.reconnect_interval_secs.replace(secs);
        self
    }

    /// Specifies the maximum number of gossip connections with unknown peers.
    pub fn with_max_unknown_peers(mut self, n: u16) -> Self {
        self.max_unknown_peers.replace(n);
        self
    }

    /// Specifies the maximum number of gossip connections with discovered peers.
    pub fn with_max_discovered_peers(mut self, n: u16) -> Self {
        self.max_discovered_peers.replace(n);
        self
    }

    /// Builds the network config.
    pub fn finish(self) -> Result<GossipLayerConfig, GossipLayerConfigError> {
        Ok(GossipLayerConfig {
            bind_addr: self
                .bind_multiaddr
                // Panic:
                // We made sure that the default is parsable.
                .unwrap_or_else(|| BIND_ADDR_DEFAULT.parse().unwrap()),
            reconnect_interval: Duration::from_secs(
                self.reconnect_interval_secs
                    .unwrap_or_else(|| RECONNECT_INTERVAL_DEFAULT.as_secs()),
            ),
            max_unknown_peers: self.max_unknown_peers.unwrap_or(MAX_UNKNOWN_PEERS_DEFAULT),
            max_discovered_peers: self.max_discovered_peers.unwrap_or(MAX_DISCOVERED_PEERS_DEFAULT),
            manual_peers: self.peering.finish()?.peers,
        })
    }
}

/// An in-memory network config builder, that becomes useful as part of integration testing.
#[cfg(test)]
#[derive(Default)]
#[must_use]
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
    #[must_use]
    pub fn finish(self) -> GossipLayerConfig {
        const DEFAULT_BIND_MULTIADDR_MEM: &str = "/memory/0";

        GossipLayerConfig {
            bind_addr: self
                .bind_multiaddr
                .unwrap_or_else(|| DEFAULT_BIND_MULTIADDR_MEM.parse().unwrap()),
            reconnect_interval: RECONNECT_INTERVAL_DEFAULT,
            max_unknown_peers: MAX_UNKNOWN_PEERS_DEFAULT,
            max_discovered_peers: MAX_DISCOVERED_PEERS_DEFAULT,
            manual_peers: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct ManualPeeringConfig {
    pub peers: Vec<PeerConfig>,
}

#[derive(Clone)]
pub struct PeerConfig {
    pub peer_id: PeerId,
    pub peer_addr: Multiaddr,
    pub peer_alias: Option<String>,
}

impl Eq for PeerConfig {}
impl PartialEq for PeerConfig {
    fn eq(&self, other: &Self) -> bool {
        self.peer_id.eq(&other.peer_id)
    }
}
impl hash::Hash for PeerConfig {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.peer_id.hash(state)
    }
}

#[derive(Default, Deserialize)]
#[must_use]
pub struct ManualPeeringConfigBuilder {
    pub peers: Option<Vec<PeerBuilder>>,
}

impl ManualPeeringConfigBuilder {
    pub fn finish(self) -> Result<ManualPeeringConfig, GossipLayerConfigError> {
        let peers = match self.peers {
            None => Default::default(),
            Some(peer_builders) => {
                // NOTE: Switch back to combinators once `map_while` is stable.

                let mut peers = HashSet::with_capacity(peer_builders.len());

                for builder in peer_builders {
                    let (multiaddr, peer_id) = split_multiaddr(&builder.multiaddr)?;
                    if !peers.insert(PeerConfig {
                        peer_id,
                        peer_addr: multiaddr,
                        peer_alias: builder.alias,
                    }) {
                        return Err(GossipLayerConfigError::DuplicateManualPeer(peer_id));
                    }
                }

                peers.into_iter().collect()
            }
        };

        Ok(ManualPeeringConfig { peers })
    }
}

fn split_multiaddr(multiaddr: &str) -> Result<(Multiaddr, PeerId), GossipLayerConfigError> {
    let mut multiaddr: Multiaddr = multiaddr
        .parse()
        .map_err(|_| GossipLayerConfigError::ParsingFailed(multiaddr.to_string()))?;

    if let Protocol::P2p(multihash) = multiaddr.pop().ok_or(GossipLayerConfigError::MultiaddrUnderspecified)? {
        Ok((
            multiaddr,
            libp2p_core::PeerId::from_multihash(multihash)
                .expect("Invalid peer Multiaddr: Make sure your peer's Id is complete.")
                .into(),
        ))
    } else {
        Err(GossipLayerConfigError::MissingP2pProtocol)
    }
}

#[derive(Deserialize)]
#[must_use]
pub struct PeerBuilder {
    #[serde(alias = "address")]
    multiaddr: String,
    alias: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_default_network_config() {
        let config = GossipLayerConfig::default();

        assert_eq!(
            config.bind_multiaddr(),
            &BIND_ADDR_DEFAULT.parse::<Multiaddr>().unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn create_with_builder_and_too_short_bind_address() {
        let _config = GossipLayerConfig::build()
            .with_bind_multiaddr("/ip4/127.0.0.1".parse().unwrap())
            .unwrap()
            .finish();
    }

    #[test]
    #[should_panic]
    fn create_with_builder_and_too_long_bind_address() {
        let _config = GossipLayerConfig::build()
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
        let _config = GossipLayerConfig::build()
            .with_bind_multiaddr("/ip4/127.0.0.1/tcp/1337".parse().unwrap())
            .unwrap()
            .finish();
    }

    #[test]
    fn create_with_builder_and_valid_dns_bind_address() {
        let _config = GossipLayerConfig::build()
            .with_bind_multiaddr("/dns/localhost/tcp/1337".parse().unwrap())
            .unwrap()
            .finish();
    }

    #[test]
    #[should_panic]
    fn create_with_mem_builder_and_non_mem_multiaddr() {
        let _config = GossipLayerConfig::build_in_memory()
            .with_bind_multiaddr("/ip4/127.0.0.1/tcp/1337".parse().unwrap())
            .finish();
    }

    #[test]
    fn create_with_mem_builder_and_valid_mem_multiaddr() {
        let _config = GossipLayerConfig::build_in_memory()
            .with_bind_multiaddr("/memory/1337".parse().unwrap())
            .finish();
    }
}
