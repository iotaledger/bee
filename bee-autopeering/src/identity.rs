// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::hash;

use crypto::signatures::ed25519::{PublicKey, SecretKey as PrivateKey, Signature};

use std::{
    convert::TryInto,
    fmt,
    sync::{Arc, RwLock},
};

const INTERNAL_ID_LENGTH: usize = 8;

/// A type that represents a local identity, which is able to sign messages.
#[derive(Clone)]
pub struct LocalId {
    private_key: Arc<RwLock<PrivateKey>>,
    peer_id: PeerId,
}

impl LocalId {
    /// Creates a new local identity.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a local identity from a 'base58' encoded ED25519 private key.
    pub fn from_bs58_encoded_private_key(private_key: impl AsRef<str>) -> Self {
        let private_key = bs58::decode(private_key.as_ref())
            .into_vec()
            .expect("error restoring private key");
        if private_key.len() != 32 {
            panic!("error restoring private key");
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&private_key[..32]);

        let private_key = PrivateKey::from_bytes(bytes);

        let public_key = private_key.public_key();
        let peer_id = PeerId::from_public_key(public_key);

        Self {
            private_key: Arc::new(RwLock::new(private_key)),
            peer_id,
        }
    }

    /// Returns the public key of this identity.
    pub fn public_key(&self) -> PublicKey {
        self.peer_id.public_key()
    }

    /// Signs a message using the private key.
    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.private_key.read().expect("error getting the lock").sign(msg)
    }
}

impl Default for LocalId {
    fn default() -> Self {
        let private_key = PrivateKey::generate().expect("error generating private key");
        let identity = PeerId::from_public_key(private_key.public_key());

        Self {
            private_key: Arc::new(RwLock::new(private_key)),
            peer_id: identity,
        }
    }
}

impl Eq for LocalId {}
impl PartialEq for LocalId {
    fn eq(&self, other: &Self) -> bool {
        self.peer_id == other.peer_id
    }
}

impl fmt::Debug for LocalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalId").field("identity", &self.peer_id).finish()
    }
}

impl fmt::Display for LocalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.peer_id)
    }
}

/// A type that represents the unique identity of a peer in the network.
#[derive(Clone)]
pub struct PeerId {
    internal_id: [u8; INTERNAL_ID_LENGTH],
    public_key: Arc<RwLock<PublicKey>>,
}

impl PeerId {
    /// Creates an identity from an ED25519 public key.
    pub fn from_public_key(public_key: PublicKey) -> Self {
        let internal_id = *&hash::sha256(&public_key.to_bytes())[..INTERNAL_ID_LENGTH]
            .try_into()
            .expect("error creating internal id");

        Self {
            internal_id,
            public_key: Arc::new(RwLock::new(public_key)),
        }
    }

    /// Returns a copy of the public key of this identity.
    pub fn public_key(&self) -> PublicKey {
        let guard = self.public_key.read().expect("error getting the lock");
        let bytes = guard.as_ref();
        // PANIC: unwrap is safe, because only valid public keys are stored.
        PublicKey::try_from_bytes(bytes.try_into().unwrap()).unwrap()
    }

    /// Returns the corresponding `libp2p::PeerId`.
    pub fn libp2p_peer_id(&self) -> libp2p_core::PeerId {
        let bytes = self.public_key.read().unwrap().to_bytes();
        let pubkey = libp2p_core::PublicKey::Ed25519(
            libp2p_core::identity::ed25519::PublicKey::decode(&bytes)
                .expect("error decoding ed25519 public key from bytes"),
        );
        libp2p_core::PeerId::from_public_key(pubkey)
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
        self.internal_id == other.internal_id
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = &bs58::encode(&self.internal_id).into_string();

        f.debug_struct("PeerId")
            .field("id", &s)
            .field(
                "public_key",
                &bs58::encode(self.public_key.read().expect("error getting the lock").as_ref()).into_string(),
            )
            .finish()
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", bs58::encode(&self.internal_id).into_string())
    }
}

impl Into<libp2p_core::PeerId> for PeerId {
    fn into(self) -> libp2p_core::PeerId {
        let PeerId {
            internal_id: id,
            public_key,
        } = self;
        let bytes = public_key.read().unwrap().to_bytes();
        let pubkey = libp2p_core::PublicKey::Ed25519(
            libp2p_core::identity::ed25519::PublicKey::decode(&bytes)
                .expect("error decoding ed25519 public key from bytes"),
        );
        libp2p_core::PeerId::from_public_key(pubkey)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multiaddr::from_base58_to_pubkey;

    impl PeerId {
        pub fn new() -> Self {
            todo!("random")
        }
    }

    #[test]
    fn create_peer_id_from_pubkey() {
        let base58_pubkey = "4H6WV54tB29u8xCcEaMGQMn37LFvM1ynNpp27TTXaqNM";
        let pubkey = from_base58_to_pubkey(base58_pubkey);

        let peer_id = PeerId::from_public_key(pubkey);
        let _ = peer_id.libp2p_peer_id();
    }
}
