// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Allows peers in the same IOTA network to exchange gossip messages with each other.

#![warn(missing_docs)]

mod config;
mod gossip;
mod local;
mod manager;
mod peer;
mod server;
mod task;
mod time;

pub mod init;

// #[cfg(test)]
// mod tests;

// Always exported
pub use crate::peer::{
    peer_data::{PeerInfo, PeerRelation, PeerType},
    peer_id::PeerId,
};

#[doc(inline)]
pub use libp2p_core::multiaddr::{Multiaddr, Protocol};

// Exported only with "full" feature flag.
#[cfg(feature = "full")]
#[doc(inline)]
pub use libp2p_core::identity::ed25519::{
    Keypair as Ed25519Keypair, PublicKey as Ed25519PublicKey, SecretKey as Ed25519SecretKey,
};

#[cfg(feature = "full")]
pub use crate::{
    config::{GossipLayerConfig, GossipLayerConfigBuilder},
    manager::{
        workers::GossipManager, GossipManagerCommand, GossipManagerCommandTx, GossipManagerEvent, GossipManagerEventRx,
        GossipRx, GossipTx,
    },
};
