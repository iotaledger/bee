// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Networking layer for the Bee framework.

#![warn(missing_docs)]
#![deny(warnings)]

mod config;
mod host;
mod network;
mod peers;
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
pub use host::{connections::Origin, Host};
pub use network::NetworkController;
pub use peers::{PeerInfo, PeerRelation};
pub use service::{commands::Command, events::Event, Service};
pub use swarm::protocols::gossip::{GossipReceiver, GossipSender};

/// A type that receives any event published by the networking layer.
pub type NetworkListener = UnboundedReceiver<Event>;

use bee_runtime::node::{Node, NodeBuilder};

use host::HostConfig;
use peers::{BannedAddrList, BannedPeerList, PeerList};
use service::{
    commands,
    events::{self, InternalEvent},
    ServiceConfig,
};

use libp2p::identity;
use log::{info, *};
use tokio::sync::mpsc::UnboundedReceiver;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use config::DEFAULT_RECONNECT_INTERVAL_SECS;

pub(crate) static RECONNECT_INTERVAL_SECS: AtomicU64 = AtomicU64::new(DEFAULT_RECONNECT_INTERVAL_SECS);
pub(crate) static NETWORK_ID: AtomicU64 = AtomicU64::new(0);
pub(crate) static MAX_UNKNOWN_PEERS: AtomicUsize = AtomicUsize::new(0);

/// Initializes the networking layer.
pub async fn init<N: Node>(
    config: NetworkConfig,
    local_keys: Keypair,
    network_id: u64,
    max_unknown_peers: usize,
    mut node_builder: N::Builder,
) -> (N::Builder, NetworkListener) {
    let NetworkConfig {
        bind_address,
        reconnect_interval_secs,
    } = config;

    RECONNECT_INTERVAL_SECS.swap(reconnect_interval_secs, Ordering::Relaxed);
    NETWORK_ID.swap(network_id, Ordering::Relaxed);
    MAX_UNKNOWN_PEERS.swap(max_unknown_peers, Ordering::Relaxed);

    let local_keys = identity::Keypair::Ed25519(local_keys);
    let local_id = PeerId::from_public_key(local_keys.public());
    info!("Local peer id: {}", local_id);

    let (command_sender, command_receiver) = commands::command_channel();
    let (internal_command_sender, internal_command_receiver) = commands::command_channel();

    let (event_sender, event_receiver) = events::event_channel::<Event>();
    let (internal_event_sender, internal_event_receiver) = events::event_channel::<InternalEvent>();

    let banned_addrs = BannedAddrList::new();
    let banned_peers = BannedPeerList::new();
    let peerlist = PeerList::new();

    let host_config = HostConfig {
        local_keys: local_keys.clone(),
        bind_address,
        peerlist: peerlist.clone(),
        banned_addrs: banned_addrs.clone(),
        banned_peers: banned_peers.clone(),
        internal_event_sender: internal_event_sender.clone(),
        internal_command_receiver,
    };

    let service_config = ServiceConfig {
        local_keys,
        peerlist,
        banned_addrs,
        banned_peers,
        event_sender: event_sender.clone(),
        internal_event_sender,
        internal_command_sender,
        command_receiver,
        internal_event_receiver,
    };

    let network_controller = NetworkController::new(command_sender, local_id);

    node_builder = node_builder
        .with_worker_cfg::<Service>(service_config)
        .with_worker_cfg::<Host>(host_config)
        .with_resource(network_controller);

    (node_builder, event_receiver)
}

/// Creates a (shorter) peer alias from a peer id.
#[macro_export]
macro_rules! alias {
    ($peer_id:expr) => {
        &$peer_id.to_base58()[46..]
    };
}
