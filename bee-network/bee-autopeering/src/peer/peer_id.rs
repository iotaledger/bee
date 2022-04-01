// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module for creating peer identities.

use std::{
    fmt,
    hash::{Hash, Hasher},
};

use crypto::signatures::ed25519::{PublicKey, SecretKey as PrivateKey, PUBLIC_KEY_LENGTH};
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};

use crate::hash;

const DISPLAY_LENGTH: usize = 16;
const DISPLAY_OFFSET: usize = 8;

/// Represents the unique identity of a peer in the network.
#[derive(Copy, Clone)]
pub struct PeerId {
    // The wrapped ED25519 public key actually representing the ID.
    public_key: PublicKey,
    // The corresponding SHA256 hash of the ED25519 public key.
    id_bytes: [u8; hash::SHA256_LEN],
}

impl PeerId {
    /// Generates a new random `PeerId`.
    pub fn generate() -> Self {
        let private_key = PrivateKey::generate().expect("error generating private key");

        Self::from_public_key(private_key.public_key())
    }

    /// Creates a peer identity from an ED25519 public key.
    pub fn from_public_key(public_key: PublicKey) -> Self {
        let id_bytes = hash::data_hash(public_key.as_ref());

        Self { id_bytes, public_key }
    }

    /// Returns the public key associated with this identity.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Returns the actual bytes representing this id.
    pub fn id_bytes(&self) -> &[u8; hash::SHA256_LEN] {
        &self.id_bytes
    }

    /// Creates the corresponding `libp2p::PeerId`.
    pub fn libp2p_peer_id(&self) -> libp2p_core::PeerId {
        libp2p_peer_id(self.public_key())
    }
}

/// Creates the corresponding `libp2p_core::PeerId` from a crypto.rs ED25519 public key.
pub fn libp2p_peer_id(public_key: &PublicKey) -> libp2p_core::PeerId {
    libp2p_core::PeerId::from_public_key(&libp2p_public_key(public_key))
}

/// Creates the corresponding `libp2p_core::PublicKey` from a crypto.rs ED25519 public key.
pub fn libp2p_public_key(public_key: &PublicKey) -> libp2p_core::PublicKey {
    libp2p_core::PublicKey::Ed25519(
        libp2p_core::identity::ed25519::PublicKey::decode(public_key.as_ref())
            .expect("error decoding ed25519 public key from bytes"),
    )
}

impl Eq for PeerId {}
impl PartialEq for PeerId {
    fn eq(&self, other: &Self) -> bool {
        self.id_bytes == other.id_bytes
    }
}

impl Hash for PeerId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id_bytes.hash(state);
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = &bs58::encode(&self.id_bytes).into_string();

        f.debug_struct("PeerId")
            .field("public_key", &bs58::encode(self.public_key).into_string())
            .field("id", &s)
            .finish()
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.libp2p_peer_id().to_base58()[DISPLAY_OFFSET..DISPLAY_OFFSET + DISPLAY_LENGTH].fmt(f)
    }
}

impl AsRef<PeerId> for PeerId {
    fn as_ref(&self) -> &PeerId {
        self
    }
}

impl AsRef<[u8]> for PeerId {
    fn as_ref(&self) -> &[u8] {
        self.public_key.as_ref()
    }
}

#[cfg(feature = "sled")]
impl From<&PeerId> for sled::IVec {
    fn from(peer: &PeerId) -> Self {
        let bytes = peer.public_key().to_bytes();
        sled::IVec::from_iter(bytes.into_iter())
    }
}

impl From<&PeerId> for libp2p_core::PeerId {
    fn from(peer_id: &PeerId) -> Self {
        libp2p_peer_id(peer_id.public_key())
    }
}

impl Serialize for PeerId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut this = serializer.serialize_struct("PeerId", 2)?;
        this.serialize_field("public_key", &self.public_key.to_bytes())?;
        this.end()
    }
}

impl<'de> Deserialize<'de> for PeerId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct("PeerId", &["public_key"], PeerIdVisitor {})
    }
}

struct PeerIdVisitor {}

impl<'de> Visitor<'de> for PeerIdVisitor {
    type Value = PeerId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("'PeerId'")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let bytes = seq
            .next_element::<[u8; PUBLIC_KEY_LENGTH]>()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

        let public_key = PublicKey::try_from_bytes(bytes).map_err(|_| serde::de::Error::invalid_length(0, &self))?;

        Ok(PeerId::from_public_key(public_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multiaddr::base58_to_pubkey;

    impl PeerId {
        /// Creates a static peer id.
        pub(crate) fn new_static() -> Self {
            let base58_pubkey = "4H6WV54tB29u8xCcEaMGQMn37LFvM1ynNpp27TTXaqNM";
            let pubkey = base58_to_pubkey(base58_pubkey).unwrap();
            Self::from_public_key(pubkey)
        }

        /// Creates a deterministic peer id from a generator char.
        pub fn new_deterministic(gen: char) -> Self {
            let base58_pubkey = std::iter::repeat(gen).take(44).collect::<String>();
            let pubkey = base58_to_pubkey(base58_pubkey).unwrap();
            Self::from_public_key(pubkey)
        }
    }

    #[test]
    fn into_libp2p_peer_id() {
        let peer_id = PeerId::new_static();
        let _ = peer_id.libp2p_peer_id();
    }
}
