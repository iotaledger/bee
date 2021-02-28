// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Networking layer for the Bee framework.

#![warn(missing_docs)]
// #![deny(warnings)]
#![allow(clippy::module_inception)]

mod config;
mod host;
mod peer;
mod service;
mod swarm;

// Reexports
#[doc(inline)]
pub use libp2p::{
    core::identity::{ed25519::Keypair, PublicKey},
    multiaddr::Protocol,
    Multiaddr, PeerId,
};

// Exports
pub use config::{NetworkConfig, NetworkConfigBuilder};
pub use host::{NetworkHost, Origin};
pub use peer::{PeerInfo, PeerRelation};
pub use service::{Command, Event, NetworkService, NetworkServiceController};
pub use swarm::protocols::gossip::{GossipReceiver, GossipSender};

/// A type that receives any event published by the networking layer.
pub type NetworkListener = UnboundedReceiver<Event>;

use bee_runtime::node::{Node, NodeBuilder};

use host::NetworkHostConfig;
use peer::PeerList;
use service::{HostCommand, NetworkServiceConfig, SwarmEvent};

use libp2p::identity;
use log::{info, *};
use tokio::sync::mpsc::UnboundedReceiver;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use config::DEFAULT_RECONNECT_INTERVAL_SECS;

pub(crate) static RECONNECT_INTERVAL_SECS: AtomicU64 = AtomicU64::new(DEFAULT_RECONNECT_INTERVAL_SECS);
pub(crate) static NETWORK_ID: AtomicU64 = AtomicU64::new(0);
pub(crate) static MAX_UNKNOWN_PEERS: AtomicUsize = AtomicUsize::new(0);
pub(crate) static MAX_DISCOVERED_PEERS: AtomicUsize = AtomicUsize::new(0);

/// Initializes the networking layer.
pub async fn init<N: Node>(
    config: NetworkConfig,
    local_keys: Keypair,
    network_id: u64,
    mut node_builder: N::Builder,
) -> (N::Builder, NetworkListener) {
    let NetworkConfig {
        bind_address,
        reconnect_interval_secs,
        max_unknown_peers_accepted,
        max_discovered_peers_dialed,
        peers,
    } = config;

    RECONNECT_INTERVAL_SECS.swap(reconnect_interval_secs, Ordering::Relaxed);
    NETWORK_ID.swap(network_id, Ordering::Relaxed);
    MAX_UNKNOWN_PEERS.swap(max_unknown_peers_accepted, Ordering::Relaxed);
    MAX_DISCOVERED_PEERS.swap(max_discovered_peers_dialed, Ordering::Relaxed);

    let local_keys = identity::Keypair::Ed25519(local_keys);
    let local_peer_id = PeerId::from_public_key(local_keys.public());
    info!("Local peer id: {}", local_peer_id);

    let (command_sender, command_receiver) = service::command_channel::<Command>();
    let (host_command_sender, host_command_receiver) = service::command_channel::<HostCommand>();

    let (event_sender, event_receiver) = service::event_channel::<Event>();
    let (swarm_event_sender, swarm_event_receiver) = service::event_channel::<SwarmEvent>();

    let peerlist = PeerList::new(local_peer_id);

    let host_config = NetworkHostConfig {
        local_keys: local_keys.clone(),
        bind_address,
        peerlist: peerlist.clone(),
        swarm_event_sender: swarm_event_sender.clone(),
        host_command_receiver,
    };

    let service_config = NetworkServiceConfig {
        local_keys,
        peerlist,
        event_sender,
        command_receiver,
        swarm_event_sender,
        swarm_event_receiver,
        host_command_sender,
    };

    let network_service_controller = NetworkServiceController::new(command_sender);

    for peer in peers {
        network_service_controller
            .send(Command::AddPeer {
                peer_id: peer.peer_id,
                address: peer.address,
                alias: peer.alias,
                relation: PeerRelation::Known,
            })
            .expect("network service command receiver dropped");
    }

    node_builder = node_builder
        .with_worker_cfg::<NetworkService>(service_config)
        .with_worker_cfg::<NetworkHost>(host_config)
        .with_resource(network_service_controller);

    (node_builder, event_receiver)
}

/// Creates a (shorter) peer alias from a peer id.
#[macro_export]
macro_rules! alias {
    ($peer_id:expr) => {
        &$peer_id.to_base58()[46..]
    };
}
