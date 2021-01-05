// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]
// #![deny(warnings)]

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
pub use interaction::{
    commands::{self, Command},
    events::{self, Event},
};
pub use network::NetworkController;
pub use peers::PeerRelation;

pub type NetworkListener = UnboundedReceiver<Event>;

use config::DEFAULT_RECONNECT_INTERVAL_SECS;
use conns::{ConnectionManager, ConnectionManagerConfig};
use interaction::events::InternalEvent;
use peers::{BannedAddrList, BannedPeerList, PeerList, PeerManager, PeerManagerConfig};

use bee_common_pt2::node::{Node, NodeBuilder};

use libp2p::identity;
use log::info;
use tokio::sync::mpsc::UnboundedReceiver;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub(crate) static RECONNECT_INTERVAL_SECS: AtomicU64 = AtomicU64::new(DEFAULT_RECONNECT_INTERVAL_SECS);
pub(crate) static NETWORK_ID: AtomicU64 = AtomicU64::new(0);
pub(crate) static MAX_UNKNOWN_PEERS: AtomicUsize = AtomicUsize::new(0);

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

    let network_controller = NetworkController::new(config, command_sender, local_id);

    node_builder = node_builder
        .with_worker_cfg::<PeerManager>(peer_manager_config)
        .with_worker_cfg::<ConnectionManager>(conn_manager_config)
        .with_resource(network_controller);

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
    const LEADING_LENGTH: usize = 0;
    const TRAILING_LENGTH: usize = 6;

    fn short(&self) -> String {
        let s = self.to_string();
        s[(Self::ORIGINAL_LENGTH - Self::TRAILING_LENGTH)..].to_string()
        // format!(
        //     // "{}~{}",
        //     // &s[..Self::LEADING_LENGTH],
        //     "{}",
        //     &s[(Self::ORIGINAL_LENGTH - Self::TRAILING_LENGTH)..]
        // )
    }
}
