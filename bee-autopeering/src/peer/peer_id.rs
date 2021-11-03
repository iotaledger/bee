// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{hash, local::salt::Salt};

use crypto::signatures::ed25519::{PublicKey, SecretKey as PrivateKey, Signature, PUBLIC_KEY_LENGTH};
use ring::signature::KeyPair;
use serde::{de::Visitor, ser::SerializeStruct, Deserialize, Serialize, Serializer};

use std::{
    convert::TryInto,
    fmt,
    hash::{Hash, Hasher},
    sync::{Arc, RwLock},
};

const DISPLAY_LENGTH: usize = 12;

/// A type that represents the unique identity of a peer in the network.
#[derive(Copy, Clone)]
pub struct PeerId {
    // An ED25519 public key.
    public_key: PublicKey,
    // The `SHA256` hash of the public key.
    id_bytes: [u8; hash::SHA256_LEN],
}

impl PeerId {
    /// Creates a new peer identity.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a peer identity from an ED25519 public key.
    pub fn from_public_key(public_key: PublicKey) -> Self {
        let id_bytes = hash::sha256(public_key.as_ref());

        Self { id_bytes, public_key }
    }

    /// Returns a copy of the public key of this identity.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn libp2p_public_key(&self) -> libp2p_core::PublicKey {
        libp2p_core::PublicKey::Ed25519(
            libp2p_core::identity::ed25519::PublicKey::decode(self.public_key.as_ref())
                .expect("error decoding ed25519 public key from bytes"),
        )
    }

    /// Returns the corresponding `libp2p::PeerId`.
    pub fn libp2p_peer_id(&self) -> libp2p_core::PeerId {
        libp2p_core::PeerId::from_public_key(self.libp2p_public_key())
    }

    /// Returns the actual bytes representing this id.
    pub fn id_bytes(&self) -> &[u8; hash::SHA256_LEN] {
        &self.id_bytes
    }
}

impl Default for PeerId {
    fn default() -> Self {
        let private_key = PrivateKey::generate().expect("error generating private key");
        Self::from_public_key(private_key.public_key())
    }
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
            .field("id", &s)
            .field("public_key", &bs58::encode(self.public_key).into_string())
            .finish()
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &bs58::encode(&self.id_bytes).into_string()[..DISPLAY_LENGTH])
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

impl Into<sled::IVec> for PeerId {
    fn into(self) -> sled::IVec {
        let bytes = self.public_key.to_bytes();
        sled::IVec::from_iter(bytes.into_iter())
    }
}

impl Into<libp2p_core::PeerId> for PeerId {
    fn into(self) -> libp2p_core::PeerId {
        let PeerId {
            id_bytes: id,
            public_key,
        } = self;

        let public_key = libp2p_core::PublicKey::Ed25519(
            libp2p_core::identity::ed25519::PublicKey::decode(public_key.as_ref())
                .expect("error decoding ed25519 public key from bytes"),
        );

        libp2p_core::PeerId::from_public_key(public_key)
    }
}

impl Serialize for PeerId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut this = serializer.serialize_struct("PeerId", 2)?;
        this.serialize_field("public_key", &self.public_key.to_bytes())?;
        this.serialize_field("id_bytes", &self.id_bytes)?;
        this.end()
    }
}

impl<'de> Deserialize<'de> for PeerId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct("PeerId", &["public_key", "id_bytes"], PeerIdVisitor {})
    }
}

struct PeerIdVisitor {}

impl<'de> Visitor<'de> for PeerIdVisitor {
    type Value = PeerId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a peer id")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multiaddr::from_base58_to_pubkey;

    impl PeerId {
        /// Creates a static peer id.
        pub(crate) fn new_static() -> Self {
            let base58_pubkey = "4H6WV54tB29u8xCcEaMGQMn37LFvM1ynNpp27TTXaqNM";
            let pubkey = from_base58_to_pubkey(base58_pubkey);
            Self::from_public_key(pubkey)
        }

        /// Creates a deterministic peer id from a generator char.
        pub fn new_deterministic(gen: char) -> Self {
            let base58_pubkey = std::iter::repeat(gen).take(44).collect::<String>();
            let pubkey = from_base58_to_pubkey(base58_pubkey);
            Self::from_public_key(pubkey)
        }
    }

    #[test]
    fn to_libp2p_peer_id() {
        let peer_id = PeerId::new_static();
        let _ = peer_id.libp2p_peer_id();
    }
}
