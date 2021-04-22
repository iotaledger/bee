// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Networking layer for the Bee framework.

#![warn(missing_docs)]

mod alias;
mod config;
mod init;
mod network;
mod peer;
mod service;
mod swarm;
mod types;

pub mod util;

// Always exported
pub use self::types::{PeerInfo, PeerRelation};
#[doc(inline)]
pub use libp2p_core::{
    multiaddr::{Multiaddr, Protocol},
    PeerId,
};

// Exported only with "full" feature flag.
#[cfg(feature = "full")]
#[doc(inline)]
pub use libp2p::core::identity::{ed25519::Keypair, PublicKey};

#[cfg(feature = "full")]
pub use crate::{
    config::{NetworkConfig, NetworkConfigBuilder},
    network::meta::Origin,
    service::{
        command::Command,
        controller::{NetworkCommandSender, NetworkEventReceiver},
        event::Event,
    },
    swarm::protocols::gossip::{GossipReceiver, GossipSender},
};

// Exported only with "standalone" feature flag.
#[cfg(feature = "standalone")]
pub use crate::init::standalone_init as init;

// Exported only with "integrated" feature flag.
#[cfg(feature = "integrated")]
pub use crate::{init::integrated_init as init, network::host::NetworkHost, service::service::NetworkService};
