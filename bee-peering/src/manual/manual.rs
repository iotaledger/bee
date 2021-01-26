// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{manual::config::ManualPeeringConfig, PeerManager};

use bee_network::{Command::*, Multiaddr, NetworkController, PeerId, PeerRelation, Protocol};

use async_trait::async_trait;
use log::warn;

// Manages a peer list and watches a config file for changes
// Sends changes (peer added/removed) to the network

#[derive(Default)]
pub struct ManualPeerManager {}

#[async_trait]
impl PeerManager for ManualPeerManager {
    type Config = ManualPeeringConfig;

    async fn new(config: Self::Config, network: &NetworkController) -> Self {
        for (mut address, alias) in config.peers {
            // NOTE: `unwrap`ing should be fine here since it comes from the config.
            if let Protocol::P2p(multihash) = address.pop().unwrap() {
                let id = PeerId::from_multihash(multihash).expect("Invalid Multiaddr.").into();

                add_peer(network, id, address, alias);
            } else {
                unreachable!(
                    "Invalid Peer descriptor. The multiaddress did not have a valid peer id as its last segment."
                )
            }
        }

        ManualPeerManager {}
    }

    async fn run(self, _: &NetworkController) {
        // TODO config file watcher
    }
}

#[inline]
fn add_peer(network: &NetworkController, id: PeerId, address: Multiaddr, alias: Option<String>) {
    if let Err(e) = network.send(AddPeer {
        id,
        address,
        alias,
        relation: PeerRelation::Known,
    }) {
        warn!("Failed to add peer: {}", e);
    }
}
