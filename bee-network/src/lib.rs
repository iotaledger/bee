// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Networking functionality and types for nodes and clients participating in the IOTA protocol built on top of
//! `libp2p`.

#![deny(missing_docs, warnings)]

mod peer;

// Exports
pub use self::peer::{PeerInfo, PeerRelation};

// Re-Exports
#[cfg(feature = "node")]
#[doc(inline)]
pub use libp2p::core::identity::{ed25519::Keypair, PublicKey};

#[doc(inline)]
pub use libp2p_core::{multiaddr::Protocol, Multiaddr, PeerId};

/// Creates a (shorter) peer alias from a peer id.
#[macro_export]
macro_rules! alias {
    ($peer_id:expr) => {
        &$peer_id.to_base58()[46..]
    };
}
