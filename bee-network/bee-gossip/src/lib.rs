// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Allows peers in the same IOTA network to exchange gossip messages with each other.

#![cfg_attr(doc_cfg, feature(doc_cfg))]
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
// Exported only with "full" feature flag.
#[cfg(feature = "full")]
#[doc(inline)]
pub use libp2p_core::identity::{
    ed25519::{Keypair, SecretKey},
    PublicKey,
};
#[doc(inline)]
pub use libp2p_core::{
    multiaddr::{Multiaddr, Protocol},
    PeerId,
};

pub use self::peer::info::{PeerInfo, PeerRelation};
#[cfg(feature = "full")]
pub use self::{
    config::{NetworkConfig, NetworkConfigBuilder},
    error::Error,
    init::{integrated, standalone},
    network::{host::integrated::NetworkHost, origin::Origin},
    service::{
        command::{Command, NetworkCommandSender},
        event::{Event, NetworkEventReceiver},
        host::integrated::ServiceHost,
    },
    swarm::protocols::iota_gossip::{GossipReceiver, GossipSender},
};
