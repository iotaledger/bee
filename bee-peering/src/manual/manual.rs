// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{manual::config::ManualPeeringConfig, PeerManager};

use bee_network::{Command::*, Multiaddr, Network, PeerId, PeerRelation, Protocol};

use async_trait::async_trait;
use log::warn;

// Manages a peer list and watches a config file for changes
// Sends changes (peer added/removed) to the network

pub struct ManualPeerManager {
    config: ManualPeeringConfig,
}

impl ManualPeerManager {
    pub fn new(config: ManualPeeringConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PeerManager for ManualPeerManager {
    async fn run(self, network: &Network) {
        let ManualPeerManager { config } = self;

        // TODO config file watcher
        for (i, (mut address, alias)) in config.peers.into_iter().enumerate() {
            if i < config.limit {
                // NOTE: `unwrap`ping should be fine here since it comes from the config.
                if let Protocol::P2p(multihash) = address.pop().unwrap() {
                    let id = PeerId::from_multihash(multihash).expect("Invalid Multiaddr.");

                    add_peer(network, id, address, alias);
                } else {
                    unreachable!(
                        "Invalid Peer descriptor. The multiaddress did not have a valid peer id as its last segment."
                    )
                }
            } else {
                warn!("Tried to add more peers than specified in limit(={})", config.limit);
            }
        }
    }
}

fn add_peer(network: &Network, id: PeerId, address: Multiaddr, alias: Option<String>) {
    if let Err(e) = network.unbounded_send(AddPeer {
        id,
        address,
        alias,
        relation: PeerRelation::Known,
    }) {
        warn!("Failed to add peer: {}", e);
    }
}
