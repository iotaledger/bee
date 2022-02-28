// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use bee_gossip::{Keypair, PeerId, PublicKey};

#[derive(Clone, Debug)]
pub struct Local {
    /// An Ed25519 keypair.
    keypair: Keypair,
    /// The hex/base16 encoded representation of the keypair.
    encoded: String,
    /// The local peer identity.
    peer_id: PeerId,
    /// Whether the identity was newly generated.
    is_new: bool,
}

impl Local {
    pub fn from_keypair(keypair: Keypair) -> Self {
        let encoded = hex::encode(keypair.encode());
        let peer_id = PeerId::from_public_key(PublicKey::Ed25519(keypair.public()));

        Self {
            keypair,
            encoded,
            peer_id,
            is_new: false,
        }
    }

    pub(crate) fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    pub(crate) fn encoded(&self) -> &str {
        &self.encoded
    }

    #[allow(dead_code)]
    pub(crate) fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub(crate) fn is_new(&self) -> bool {
        self.is_new
    }
}

impl fmt::Display for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PeerId: {}", self.peer_id)
    }
}
