// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "integrated")]
use bee_runtime::node::{Node, NodeBuilder};

use super::{
    config::NetworkConfig,
    network::host::{NetworkHost, NetworkHostConfig},
    peer::{
        ban::{AddrBanlist, PeerBanlist},
        store::PeerList,
    },
    service::{
        command::{command_channel, Command},
        controller::NetworkServiceController,
        event::{event_channel, Event, InternalEvent},
        service::{NetworkService, NetworkServiceConfig},
    },
    Keypair, PeerId, PeerRelation,
};

use libp2p::identity;
use log::info;
use tokio::sync::mpsc::UnboundedReceiver;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use super::config::DEFAULT_RECONNECT_INTERVAL_SECS;

// TODO: use OnceCell
pub(crate) static RECONNECT_INTERVAL_SECS: AtomicU64 = AtomicU64::new(DEFAULT_RECONNECT_INTERVAL_SECS);
pub(crate) static NETWORK_ID: AtomicU64 = AtomicU64::new(0);
pub(crate) static MAX_UNKNOWN_PEERS: AtomicUsize = AtomicUsize::new(0);

/// A type that receives any event published by the networking layer.
pub type NetworkListener = UnboundedReceiver<Event>;

/// Initializes the networking layer.
#[cfg(feature = "standalone")]
#[cfg(not(feature = "integrated"))]
pub async fn init(config: NetworkConfig, local_keys: Keypair, network_id: u64) -> NetworkListener {
    todo!("headleass init")
}

/// Initializes the networking layer.
#[cfg(feature = "integrated")]
pub async fn init<N: Node>(
    config: NetworkConfig,
    local_keys: Keypair,
    network_id: u64,
    mut node_builder: N::Builder,
) -> (N::Builder, NetworkListener) {
    let NetworkConfig {
        bind_multiaddr,
        reconnect_interval_secs,
        max_unknown_peers,
        peers,
    } = config;

    RECONNECT_INTERVAL_SECS.swap(reconnect_interval_secs, Ordering::Relaxed);
    NETWORK_ID.swap(network_id, Ordering::Relaxed);
    MAX_UNKNOWN_PEERS.swap(max_unknown_peers, Ordering::Relaxed);

    let local_keys = identity::Keypair::Ed25519(local_keys);
    let local_peer_id = PeerId::from_public_key(local_keys.public());
    info!("Local peer id: {}", local_peer_id);

    let (command_sender, command_receiver) = command_channel();
    let (internal_command_sender, internal_command_receiver) = command_channel();

    let (event_sender, event_receiver) = event_channel::<Event>();
    let (internal_event_sender, internal_event_receiver) = event_channel::<InternalEvent>();

    let banned_addrs = AddrBanlist::new();
    let banned_peers = PeerBanlist::new();
    let peerlist = PeerList::new();

    let host_config = NetworkHostConfig {
        local_keys: local_keys.clone(),
        bind_multiaddr,
        peerlist: peerlist.clone(),
        banned_addrs: banned_addrs.clone(),
        banned_peers: banned_peers.clone(),
        internal_event_sender: internal_event_sender.clone(),
        internal_command_receiver,
    };

    let service_config = NetworkServiceConfig {
        local_keys,
        peerlist,
        banned_addrs,
        banned_peers,
        event_sender,
        internal_event_sender,
        internal_command_sender,
        command_receiver,
        internal_event_receiver,
    };

    let network_service_controller = NetworkServiceController::new(command_sender);

    // TODO: make this a closure that gets executed only when everything has been initialized!
    for peer in peers {
        network_service_controller
            .send(Command::AddPeer {
                peer_id: peer.peer_id,
                multiaddr: peer.multiaddr,
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
