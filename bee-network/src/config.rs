// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module containing types necessary for network (layer) configuration.

use crate::{
    identity::{Identity, LocalIdentity},
    util,
};

use thiserror::Error;
use yaml_rust::{ScanError, Yaml, YamlLoader};

use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    net::{AddrParseError, IpAddr, Ipv4Addr, SocketAddr},
};

#[derive(Clone)]
/// Network configuration.
pub struct Config {
    /// The bind address for the server accepting peers to exchange gossip.
    pub bind_addr: SocketAddr,
    /// The local identity of the node.
    pub local_id: LocalIdentity,
    /// The config to set up manual/static/known peers.
    pub manual_peer_config: ManualPeerConfig,
}

impl Config {
    /// Creates a new config.
    pub fn new(port: u16, local_id: LocalIdentity, peers_file_path: &str) -> Result<Self, ManualPeerConfigError> {
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        let manual_peer_config = ManualPeerConfig::from_file(&local_id, peers_file_path)?;

        Ok(Self {
            bind_addr,
            local_id,
            manual_peer_config,
        })
    }
}

/// Stores connection and other information about a manual peer.
#[derive(Debug, Clone)]
pub struct ManualPeerInfo {
    /// The identity of the peer.
    pub identity: Identity,
    /// The address of the peer.
    pub address: SocketAddr,
    /// A human friendly identifier of the peer.
    pub alias: String,
    /// Whether the peer is supposed to dial *us*.
    is_dialer: bool,
}

impl ManualPeerInfo {
    /// Whether the peer is supposed to be the initiator of a connection.
    pub fn is_dialer(&self) -> bool {
        self.is_dialer
    }
}

/// Error returned when the peer configuration file could not be loaded.
#[derive(Debug, Error)]
pub enum ManualPeerConfigError {
    /// The YAML file has an invalid layout.
    #[error("invalid config layout: {0}")]
    InvalidConfig(&'static str),
    /// Parsing the peer's address failed.
    #[error("address could not be parsed: {0}")]
    AddrParse(#[from] AddrParseError),
    /// Could not load the YAML file after reading it.
    #[error("YAML file could not be parsed: {0}")]
    Scan(#[from] ScanError),
    /// Could not read the YAML file.
    #[error("YAML file could not be read: {0}")]
    Io(#[from] std::io::Error),
}

/// A YAML backed database for manual peers.
#[derive(Clone)]
pub struct ManualPeerConfig {
    infos: HashMap<IpAddr, ManualPeerInfo>,
}

impl ManualPeerConfig {
    /// Restores the database from its backing file.
    pub fn from_file(local_id: &LocalIdentity, path: &str) -> Result<Self, ManualPeerConfigError> {
        let mut file = File::open(path)?;
        let mut s = String::new();

        file.read_to_string(&mut s)?;

        let mut docs = YamlLoader::load_from_str(&s)?;

        if docs.len() != 1 {
            return Err(ManualPeerConfigError::InvalidConfig("expected a single document"));
        }

        let doc = docs.remove(0);

        let peers_config = doc
            .into_vec()
            .ok_or(ManualPeerConfigError::InvalidConfig("peers YAML is not an array"))?;
        let mut infos = HashMap::with_capacity(peers_config.len());

        for m in peers_config {
            let hm = m
                .into_hash()
                .ok_or(ManualPeerConfigError::InvalidConfig("invalid YAML file"))?;

            let public_key_str = hm
                .get(&Yaml::String("public_key".into()))
                .ok_or(ManualPeerConfigError::InvalidConfig("missing `public_key` key"))?
                .as_str()
                .ok_or(ManualPeerConfigError::InvalidConfig(
                    "`public_key` value is not a string",
                ))?;

            let public_key = util::from_public_key_string(public_key_str);

            let dialer = public_key < local_id.public_key();

            let identity = Identity::from_public_key(public_key);

            let address = hm
                .get(&Yaml::String("address".into()))
                .ok_or(ManualPeerConfigError::InvalidConfig("missing `address` key"))?
                .as_str()
                .ok_or(ManualPeerConfigError::InvalidConfig("`address` value is not a string"))?
                .parse::<SocketAddr>()
                .map_err(ManualPeerConfigError::AddrParse)?;
            let ip = address.ip();

            let alias = hm
                .get(&Yaml::String("alias".into()))
                .ok_or(ManualPeerConfigError::InvalidConfig("missing `alias` key"))?
                .as_str()
                .ok_or(ManualPeerConfigError::InvalidConfig("`alias` value is not a string"))?
                .to_string();

            let peer_info = ManualPeerInfo {
                identity,
                address,
                alias,
                is_dialer: dialer,
            };

            if infos.contains_key(&ip) {
                unimplemented!("multiple instances with same ip address");
            }

            infos.insert(ip, peer_info);
        }

        Ok(Self { infos })
    }

    /// Returns a [`ManualPeerInfo`] associated with a particular [`IpAddr`].
    pub fn get(&self, ip_addr: &IpAddr) -> Option<&ManualPeerInfo> {
        self.infos.get(ip_addr)
    }

    /// Adds a new static peer.
    pub fn add(&mut self, _info: ManualPeerInfo) -> bool {
        todo!("add manual peers")
    }

    /// Iterates over all manual peers.
    pub fn iter(&self) -> impl Iterator<Item = (&IpAddr, &ManualPeerInfo)> {
        self.infos.iter()
    }
}
