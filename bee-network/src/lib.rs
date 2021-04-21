// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Networking layer for the Bee framework.

#![warn(missing_docs)]

mod config;
mod init;
mod network;
mod peer;
mod service;
mod swarm;
mod types;

// Re-Exports
#[cfg(feature = "standalone")]
#[doc(inline)]
pub use libp2p::core::identity::{ed25519::Keypair, PublicKey};

#[doc(inline)]
pub use libp2p_core::{
    multiaddr::{Multiaddr, Protocol},
    PeerId,
};

// Exports
pub use self::types::{PeerInfo, PeerRelation};

#[cfg(feature = "standalone")]
pub use crate::{
    config::{NetworkConfig, NetworkConfigBuilder},
    init::{init, NetworkListener},
    network::{host::NetworkHost, meta::Origin},
    service::{command::Command, controller::NetworkServiceController, event::Event, service::NetworkService},
    swarm::protocols::gossip::{GossipReceiver, GossipSender},
};

// TODO 1: A functional macro that ...
// TODO 2: Allow to give it a length.
// TODO 3: Consider moving it into its own module.
/// Creates a (shorter) peer alias from a peer id.
#[macro_export]
macro_rules! alias {
    ($peer_id:expr) => {
        &$peer_id.to_base58()[46..]
    };
}
