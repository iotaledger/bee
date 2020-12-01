// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{manual::config::ManualPeeringConfig, PeerManager};

use bee_network::{Command::*, Multiaddr, Network, PeerId, PeerRelation, Protocol};

use async_trait::async_trait;
use log::warn;

// Manages a peer list and watches a config file for changes
// Sends changes (peer added/removed) to the network

#[derive(Default)]
pub struct ManualPeerManager {}

#[async_trait]
impl PeerManager for ManualPeerManager {
    type Config = ManualPeeringConfig;

    async fn start(config: Self::Config, network: &Network) -> Self {
        for (i, (mut address, alias)) in config.peers.into_iter().enumerate() {
            if i < config.limit {
                // NOTE: `unwrap`ping should be fine here since it comes from the config.
                if let Protocol::P2p(multihash) = address.pop().unwrap() {
                    let id = PeerId::from_multihash(multihash).expect("Invalid Multiaddr.");

                    add_peer(network, id, address, alias).await;
                } else {
                    unreachable!(
                        "Invalid Peer descriptor. The multiaddress did not have a valid peer id as its last segment."
                    )
                }
            } else {
                warn!("Tried to add more peers than specified in limit(={})", config.limit);
            }
        }

        ManualPeerManager {}
    }

    async fn run(self, network: &Network) {
        // TODO config file watcher
    }
}

#[inline]
async fn add_peer(network: &Network, id: PeerId, address: Multiaddr, alias: Option<String>) {
    if let Err(e) = network
        .send(AddPeer {
            id,
            address,
            alias,
            relation: PeerRelation::Known,
        })
        .await
    {
        warn!("Failed to add peer: {}", e);
    }
}
