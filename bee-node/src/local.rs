// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use bee_gossip::{Keypair, PeerId, PublicKey};

use crate::{KEYPAIR_STR_LENGTH, LOCAL_ALIAS_DEFAULT};

#[derive(Debug, thiserror::Error)]
pub enum LocalError {
    #[error("invalid ed25519 keypair")]
    InvalidKeypair,
    #[error("cannot decode from hex representation")]
    HexDecode,
    #[error("cannot decode ed25519 keypair")]
    KeypairDecode,
}

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
    /// Creates a new local identity with the given alias.
    pub(crate) fn new(alias: Option<String>) -> Self {
        let keypair = Keypair::generate();
        let encoded = hex::encode(keypair.encode());
        let peer_id = PeerId::from_public_key(PublicKey::Ed25519(keypair.public()));

        Self {
            keypair,
            encoded,
            peer_id,
            alias: alias.unwrap_or_else(|| LOCAL_ALIAS_DEFAULT.to_owned()),
            is_new: true,
        }
    }

    /// Restores a local identity from a `hex`/`base16` encoded Ed25519 keypair with the given alias.
    pub(crate) fn from_keypair(encoded: String, alias: Option<String>) -> Result<Self, LocalError> {
        if encoded.len() == KEYPAIR_STR_LENGTH {
            // Decode the keypair from hex.
            let mut decoded = [0u8; 64];
            hex::decode_to_slice(&encoded[..], &mut decoded).map_err(|_| LocalError::HexDecode)?;

            // Decode the keypair from bytes.
            let keypair = Keypair::decode(&mut decoded).map_err(|_| LocalError::KeypairDecode)?;
            let peer_id = PeerId::from_public_key(PublicKey::Ed25519(keypair.public()));

            Ok(Local {
                keypair,
                encoded,
                peer_id,
                is_new: false,
                alias: alias.unwrap_or_else(|| LOCAL_ALIAS_DEFAULT.to_owned()),
            })
        } else {
            Err(LocalError::InvalidKeypair)
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
