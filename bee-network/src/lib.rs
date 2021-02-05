// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Networking layer for the Bee framework.

#![warn(missing_docs)]
#![deny(warnings)]

mod config;
mod conns;
mod interaction;
mod network;
mod peers;
mod protocols;
mod transport;

// Reexports
#[doc(inline)]
pub use libp2p::{
    core::identity::{ed25519::Keypair, PublicKey},
    multiaddr::Protocol,
    Multiaddr, PeerId,
};

// Exports
pub use config::{NetworkConfig, NetworkConfigBuilder};
pub use conns::Origin;
pub use interaction::{commands::Command, events::Event};
pub use network::NetworkController;
pub use peers::{MessageReceiver, MessageSender, NetworkService, PeerInfo, PeerRelation};

/// A type that receives any event published by the networking layer.
pub type NetworkListener = UnboundedReceiver<Event>;

use config::DEFAULT_RECONNECT_INTERVAL_SECS;
use conns::{Server, ServerConfig};
use interaction::{
    commands,
    events::{self, InternalEvent},
};
use peers::{BannedAddrList, BannedPeerList, NetworkServiceConfig, PeerList};

use bee_runtime::node::{Node, NodeBuilder};

use libp2p::identity;
use log::info;
use tokio::sync::mpsc::UnboundedReceiver;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

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
    RECONNECT_INTERVAL_SECS.swap(config.reconnect_interval_secs, Ordering::Relaxed);
    NETWORK_ID.swap(network_id, Ordering::Relaxed);
    MAX_UNKNOWN_PEERS.swap(max_unknown_peers, Ordering::Relaxed);

    let local_keys = identity::Keypair::Ed25519(local_keys);
    let local_id = PeerId::from_public_key(local_keys.public());
    info!("Own peer id: {}", local_id);

    let (command_sender, command_receiver) = commands::channel();
    let (event_sender, event_receiver) = events::channel::<Event>();
    let (internal_event_sender, internal_event_receiver) = events::channel::<InternalEvent>();

    let banned_addrs = BannedAddrList::new();
    let banned_peers = BannedPeerList::new();
    let peers = PeerList::new();

    let network_service_config = NetworkServiceConfig::new(
        local_keys.clone(),
        peers.clone(),
        banned_addrs.clone(),
        banned_peers.clone(),
        event_sender,
        internal_event_sender.clone(),
        command_receiver,
        internal_event_receiver,
    );

    let server_config = ServerConfig::new(
        local_keys,
        config.bind_address.clone(),
        peers,
        banned_addrs,
        banned_peers,
        internal_event_sender,
    )
    .await
    .unwrap_or_else(|e| {
        panic!("Fatal error: {}", e);
    });

    let network_controller = NetworkController::new(config, command_sender, local_id);

    node_builder = node_builder
        .with_worker_cfg::<NetworkService>(network_service_config)
        .with_worker_cfg::<Server>(server_config)
        .with_resource(network_controller);

    (node_builder, event_receiver)
}

/// A trait specifically there to create shorter peer ids for better readability in logs and user interfaces.
pub trait ShortId
where
    Self: ToString,
{
    /// The length of the shortened peer id.
    const SHORT_LENGTH: usize;

    /// Creates a shorter - more readable - id from the original.
    fn short(&self) -> String;
}

impl ShortId for PeerId {
    const SHORT_LENGTH: usize = 6;

    fn short(&self) -> String {
        const FULL_LENGTH: usize = 52;

        let s = self.to_string();
        s[(FULL_LENGTH - Self::SHORT_LENGTH)..].to_string()
    }
}
