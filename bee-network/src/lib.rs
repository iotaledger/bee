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
pub use libp2p::{core::identity::ed25519::Keypair, multiaddr::Protocol, Multiaddr};

// Exports
pub use config::{NetworkConfig, NetworkConfigBuilder};
pub use conns::Origin;
pub use interaction::{commands::Command, events::Event};
pub use network::NetworkController;
pub use peers::{MessageReceiver, MessageSender, PeerInfo, PeerRelation};

/// A type that receives any event published by the networking layer.
pub type NetworkListener = UnboundedReceiver<Event>;

use config::DEFAULT_RECONNECT_INTERVAL_SECS;
use conns::{ConnectionManager, ConnectionManagerConfig};
use interaction::{
    commands,
    events::{self, InternalEvent},
};
use peers::{BannedAddrList, BannedPeerList, PeerList, PeerManager, PeerManagerConfig};

use bee_runtime::node::{Node, NodeBuilder};

use libp2p::{identity, identity::PublicKey, multihash::Multihash};
use log::info;
use tokio::sync::mpsc::UnboundedReceiver;

use std::{
    fmt,
    hash::Hash,
    ops::Deref,
    str::FromStr,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
};

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
    let local_id = libp2p::PeerId::from_public_key(local_keys.public()).into();
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
    .await
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

/// A libp2p `PeerId` wrapper, that buffers its base58 representation, so that it can be viewed without extra
/// allocation.
#[derive(Clone)]
pub struct PeerId(libp2p::PeerId, Arc<String>);

impl PeerId {
    /// Builds a `PeerId` from an Ed25519 public key.
    pub fn from_public_key(ed25519_pk: identity::ed25519::PublicKey) -> Self {
        let peer_id = libp2p::PeerId::from_public_key(PublicKey::Ed25519(ed25519_pk));
        let buffer = Arc::new(peer_id.to_base58());

        Self(peer_id, buffer)
    }

    /// Tries to turn a `Multihash` into a `PeerId`.
    pub fn from_multihash(multihash: Multihash) -> Result<Self, Multihash> {
        let peer_id = libp2p::PeerId::from_multihash(multihash)?;
        let buffer = Arc::new(peer_id.to_base58());

        Ok(Self(peer_id, buffer))
    }

    /// Provides a shortened view (the last 6 characters) into the string representation of this `PeerId`.
    pub fn short(&self) -> &str {
        const SHORT_LENGTH: usize = 6;

        let len = self.1.len();
        debug_assert!(len <= 52);

        &self.1[(len - SHORT_LENGTH)..]
    }

    /// Provides the full view (52 characters) into the string representation of this `PeerId`.
    pub fn long(&self) -> &str {
        &self.1
    }
}

impl From<libp2p::PeerId> for PeerId {
    fn from(peer_id: libp2p::PeerId) -> Self {
        let base58 = peer_id.to_base58();
        Self(peer_id, Arc::new(base58))
    }
}

impl Deref for PeerId {
    type Target = libp2p::PeerId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for PeerId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(libp2p::PeerId::from_str(s).map_err(|_| "error parsing peer id")?.into())
    }
}

impl Eq for PeerId {}

impl PartialEq for PeerId {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(other)
    }
}

impl Hash for PeerId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short())
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.long())
    }
}
