// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Networking layer for the Bee framework.

#![warn(missing_docs)]

mod alias;
mod config;
mod error;
mod init;
mod network;
mod peer;
mod service;
mod swarm;

#[cfg(test)]
mod tests;

// Always exported
pub use self::peer::meta::{PeerInfo, PeerRelation};
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
    error::Error,
    init::{integrated, standalone},
    network::host::integrated::NetworkHost,
    network::meta::Origin,
    service::{
        command::{Command, NetworkCommandSender},
        event::{Event, NetworkEventReceiver},
        host::integrated::ServiceHost,
    },
    swarm::protocols::gossip::io::{GossipReceiver, GossipSender},
};
