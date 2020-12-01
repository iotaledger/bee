// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

use bee_network::Multiaddr;

use std::str::FromStr;

// TODO add acceptAnyConnection

const DEFAULT_LIMIT: usize = 5;

#[derive(Default, Deserialize)]
pub struct ManualPeeringConfigBuilder {
    pub(crate) limit: Option<usize>,
    pub(crate) peers: Option<Vec<Peer>>,
}

#[derive(Deserialize)]
pub struct Peer {
    address: String,
    alias: Option<String>,
}

impl ManualPeeringConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit.replace(limit);
        self
    }

    pub fn peers(mut self, peers: Vec<Peer>) -> Self {
        self.peers.replace(peers);
        self
    }

    pub fn finish(self) -> ManualPeeringConfig {
        let peers = match self.peers {
            None => Vec::new(),
            Some(peers) => peers
                .into_iter()
                .map(|peer| {
                    (
                        Multiaddr::from_str(&peer.address[..]).expect("error parsing multiaddr."),
                        peer.alias,
                    )
                })
                .collect(),
        };

        ManualPeeringConfig {
            limit: self.limit.unwrap_or(DEFAULT_LIMIT),
            peers,
        }
    }
}

#[derive(Clone)]
pub struct ManualPeeringConfig {
    pub(crate) limit: usize,
    pub(crate) peers: Vec<(Multiaddr, Option<String>)>,
}

impl ManualPeeringConfig {
    pub fn build() -> ManualPeeringConfigBuilder {
        ManualPeeringConfigBuilder::new()
    }
}
