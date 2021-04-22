// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use super::{
    config::NetworkConfig,
    peer::{
        ban::{AddrBanlist, PeerBanlist},
        store::PeerList,
    },
    service::{
        command::{command_channel, Command},
        controller::{NetworkCommandSender, NetworkEventReceiver},
        event::{event_channel, Event, InternalEvent},
    },
    Keypair, PeerId, PeerRelation,
};

use libp2p::identity;
use log::info;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use super::config::DEFAULT_RECONNECT_INTERVAL_SECS;

// TODO: use OnceCell
pub(crate) static RECONNECT_INTERVAL_SECS: AtomicU64 = AtomicU64::new(DEFAULT_RECONNECT_INTERVAL_SECS);
pub(crate) static NETWORK_ID: AtomicU64 = AtomicU64::new(0);
pub(crate) static MAX_UNKNOWN_PEERS: AtomicUsize = AtomicUsize::new(0);

/// Initializes the networking service.
#[cfg(feature = "standalone")]
#[cfg(not(feature = "integrated"))]
pub async fn init(
    config: NetworkConfig,
    keys: Keypair,
    network_id: u64,
) -> (NetworkCommandSender, NetworkEventReceiver) {
    todo!("standalone init")
}

#[cfg(feature = "integrated")]
use bee_runtime::node::{Node, NodeBuilder};

#[cfg(feature = "integrated")]
use crate::{
    network::host::{NetworkHost, NetworkHostConfig},
    service::service::{NetworkService, NetworkServiceConfig},
};

/// Initializes the networking service.
#[cfg(feature = "integrated")]
#[cfg(not(feature = "standalone"))]
pub async fn init<N: Node>(
    config: NetworkConfig,
    keys: Keypair,
    network_id: u64,
    mut node_builder: N::Builder,
) -> (N::Builder, NetworkEventReceiver) {
    let NetworkConfig {
        bind_multiaddr,
        reconnect_interval_secs,
        max_unknown_peers,
        peers,
    } = config;

    RECONNECT_INTERVAL_SECS.swap(reconnect_interval_secs, Ordering::Relaxed);
    NETWORK_ID.swap(network_id, Ordering::Relaxed);
    MAX_UNKNOWN_PEERS.swap(max_unknown_peers, Ordering::Relaxed);

    // TODO: Create event
    let local_keys = identity::Keypair::Ed25519(keys);
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

    let network_command_sender = NetworkCommandSender::new(command_sender);
    let network_event_receiver = NetworkEventReceiver::new(event_receiver);

    // TODO: make this a closure that gets executed only when everything has been initialized!
    for peer in peers {
        network_command_sender
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
        .with_resource(network_command_sender);

    (node_builder, network_event_receiver)
}
