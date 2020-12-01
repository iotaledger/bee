// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// #![warn(missing_docs)]
#![recursion_limit = "512"]

mod config;
mod conns;
mod interaction;
mod network;
mod peers;
mod protocols;
mod transport;

// Exports
pub use config::{NetworkConfig, NetworkConfigBuilder};
pub use conns::Origin;
pub use interaction::{
    commands::{self, Command},
    events::{self, Event},
};
#[doc(inline)]
pub use libp2p::{
    core::identity::{ed25519::Keypair, PublicKey},
    multiaddr::Protocol,
    Multiaddr, PeerId,
};
pub use network::Network;
pub use peers::PeerRelation;

pub type EventReceiver = flume::Receiver<Event>;

use config::{DEFAULT_KNOWN_PEER_LIMIT, DEFAULT_MSG_BUFFER_SIZE, DEFAULT_RECONNECT_MILLIS, DEFAULT_UNKNOWN_PEER_LIMIT};
use conns::{ConnectionManager, ConnectionManagerConfig};
use interaction::events::InternalEvent;
use peers::{BannedAddrList, BannedPeerList, PeerList, PeerManager, PeerManagerConfig};

use bee_common::node::{Node, NodeBuilder};

use libp2p::identity;
use log::info;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub(crate) static MSG_BUFFER_SIZE: AtomicUsize = AtomicUsize::new(DEFAULT_MSG_BUFFER_SIZE);
pub(crate) static RECONNECT_MILLIS: AtomicU64 = AtomicU64::new(DEFAULT_RECONNECT_MILLIS);
pub(crate) static KNOWN_PEER_LIMIT: AtomicUsize = AtomicUsize::new(DEFAULT_KNOWN_PEER_LIMIT);
pub(crate) static UNKNOWN_PEER_LIMIT: AtomicUsize = AtomicUsize::new(DEFAULT_UNKNOWN_PEER_LIMIT);
pub(crate) static NETWORK_ID: AtomicU64 = AtomicU64::new(0);

pub async fn init<N: Node>(
    config: NetworkConfig,
    local_keys: Keypair,
    network_id: u64,
    mut node_builder: N::Builder,
) -> (N::Builder, EventReceiver) {
    MSG_BUFFER_SIZE.swap(config.msg_buffer_size, Ordering::Relaxed);
    RECONNECT_MILLIS.swap(config.reconnect_millis, Ordering::Relaxed);
    KNOWN_PEER_LIMIT.swap(config.known_peer_limit, Ordering::Relaxed);
    UNKNOWN_PEER_LIMIT.swap(config.unknown_peer_limit, Ordering::Relaxed);
    NETWORK_ID.swap(network_id, Ordering::Relaxed);

    let local_keys = identity::Keypair::Ed25519(local_keys);
    let local_id = PeerId::from_public_key(local_keys.public());
    info!("Own Peer Id = {}", local_id);

    let (command_sender, command_receiver) = commands::channel();
    let (event_sender, event_receiver) = events::channel::<Event>();
    let (internal_event_sender, internal_event_receiver) = events::channel::<InternalEvent>();

    let banned_addrs = BannedAddrList::new();
    let banned_peers = BannedPeerList::new();
    let peers = PeerList::new();

    let peer_manager_config = PeerManagerConfig::new(
        local_keys.clone(),
        peers.clone(),
        banned_addrs.clone(),
        banned_peers.clone(),
        event_sender,
        internal_event_sender.clone(),
        command_receiver,
        internal_event_receiver,
    );

    let conn_manager_config = ConnectionManagerConfig::new(
        local_keys,
        config.bind_address.clone(),
        peers,
        banned_addrs,
        banned_peers,
        internal_event_sender,
    )
    .unwrap_or_else(|e| {
        panic!("Fatal error: {}", e);
    });

    let listen_address = conn_manager_config.listen_address.clone();
    let network = Network::new(config, command_sender, listen_address, local_id);

    node_builder = node_builder
        .with_worker_cfg::<PeerManager>(peer_manager_config)
        .with_worker_cfg::<ConnectionManager>(conn_manager_config)
        .with_resource(network);

    (node_builder, event_receiver)
}

pub trait ShortId
where
    Self: ToString,
{
    const ORIGINAL_LENGTH: usize;
    const LEADING_LENGTH: usize;
    const TRAILING_LENGTH: usize;

    fn short(&self) -> String;
}

impl ShortId for PeerId {
    const ORIGINAL_LENGTH: usize = 52;
    const LEADING_LENGTH: usize = 2;
    const TRAILING_LENGTH: usize = 6;

    fn short(&self) -> String {
        let s = self.to_string();
        format!(
            "{}~{}",
            &s[0..Self::LEADING_LENGTH],
            &s[(Self::ORIGINAL_LENGTH - Self::TRAILING_LENGTH)..]
        )
    }
}
