// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use bee_gossip::{Keypair, PeerId, PublicKey};

use crate::LOCAL_ALIAS_DEFAULT;

#[derive(Clone, Debug)]
pub struct Local {
    /// An Ed25519 keypair.
    keypair: Keypair,
    /// The hex/base16 encoded representation of the keypair.
    encoded: String,
    /// The local peer identity.
    peer_id: PeerId,
    /// The local alias.
    alias: String,
    /// Whether the identity was newly generated.
    is_new: bool,
}

impl Local {
    pub fn from_keypair(keypair: Keypair, alias: Option<String>) -> Self {
        let encoded = hex::encode(keypair.encode());
        let peer_id = PeerId::from_public_key(PublicKey::Ed25519(keypair.public()));

        Local {
            keypair,
            encoded,
            peer_id,
            is_new: false,
            alias: alias.unwrap_or_else(|| LOCAL_ALIAS_DEFAULT.to_owned()),
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

    #[allow(dead_code)]
    pub(crate) fn alias(&self) -> &str {
        &self.alias
    }

    pub(crate) fn is_new(&self) -> bool {
        self.is_new
    }
}

impl fmt::Display for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PeerId: {}, Alias: {}", self.peer_id, self.alias)
    }
}
