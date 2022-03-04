// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! TODO

use std::fmt;

use libp2p_core::{identity::PublicKey, PeerId};

use crate::{Keypair as Ed25519Keypair, PublicKey as Ed25519PublicKey};

/// TODO
#[derive(Clone, Debug)]
pub struct Identity {
    /// An Ed25519 keypair.
    keypair: Ed25519Keypair,
    /// The hex/base16 encoded representation of the keypair.
    encoded: String,
    /// The local peer identity.
    peer_id: PeerId,
    /// Whether the identity was newly generated.
    is_new: bool,
}

impl Identity {
    /// Restores a [`Local`] from a [`Keypair`](crate::keypair::Keypair).
    pub fn from_keypair(keypair: Ed25519Keypair) -> Self {
        let encoded = hex::encode(keypair.encode());
        let peer_id = PeerId::from_public_key(&PublicKey::Ed25519(keypair.public()));

        Self {
            keypair,
            encoded,
            peer_id,
            is_new: false,
        }
    }

    /// Returns the [`Keypair`](crate::keypair::Keypair) of this identity.
    pub fn keypair(&self) -> &Ed25519Keypair {
        &self.keypair
    }

    /// Returns the [`PublicKey`](crate::keypair::PublicKey) of this identity.
    pub fn public_key(&self) -> Ed25519PublicKey {
        self.keypair.public()
    }

    /// TODO
    pub fn encoded(&self) -> &str {
        &self.encoded
    }

    /// TODO
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// TODO
    pub fn is_new(&self) -> bool {
        self.is_new
    }
}

impl Default for Identity {
    fn default() -> Self {
        Identity::from_keypair(Ed25519Keypair::generate())
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.peer_id)
    }
}
