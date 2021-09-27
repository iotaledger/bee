// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that deals with network identities.

use crypto::{
    hashes::sha,
    signatures::ed25519::{PublicKey, SecretKey as PrivateKey, Signature},
};

use std::{
    fmt,
    sync::{Arc, RwLock},
};

/// The length in bytes of the peer id (32 bytes).
const ID_LENGTH: usize = crypto::hashes::sha::SHA256_LEN;

/// A type that represents a local identity, which is also able to sign messages.
#[derive(Clone)]
pub struct LocalId {
    private_key: Arc<RwLock<PrivateKey>>,
    identity: PeerId,
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
        let identity = PeerId::from_public_key(public_key);

        Self {
            private_key: Arc::new(RwLock::new(private_key)),
            identity,
        }
    }

    /// Returns the public key of this identity.
    pub fn public_key(&self) -> PublicKey {
        self.identity.public_key()
    }

    /// Returns the ID of this local identity.
    pub fn id_string(&self) -> String {
        self.identity.id_string()
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
            identity,
        }
    }
}

impl fmt::Debug for LocalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalIdentity")
            .field("identity", &self.identity)
            .finish()
    }
}

/// A type that represents the unique identity of a peer in the network.
#[derive(Clone)]
pub struct PeerId {
    id: [u8; ID_LENGTH],
    public_key: Arc<RwLock<PublicKey>>,
}

impl PeerId {
    /// Creates an identity from an ED25519 public key.
    pub fn from_public_key(public_key: PublicKey) -> Self {
        let id = gen_id(&public_key);
        Self {
            id,
            public_key: Arc::new(RwLock::new(public_key)),
        }
    }

    /// Returns the public key of this identity.
    pub fn public_key(&self) -> PublicKey {
        let guard = self.public_key.read().expect("error getting the lock");
        let bytes = guard.as_ref();
        let mut pk = [0u8; 32];
        pk.copy_from_slice(bytes);
        PublicKey::try_from_bytes(pk).expect("error cloning public key")
    }

    /// Returns the 'base58' string representation (created from of the first 8 bytes of the 32 byte long internal id)
    pub fn id_string(&self) -> String {
        bs58::encode(&self.id[..8]).into_string()
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = &bs58::encode(&self.id[..8]).into_string()[..];

        f.debug_struct("Identity")
            .field("id", &s)
            .field(
                "public_key",
                &bs58::encode(self.public_key.read().expect("error getting the lock").as_ref()).into_string(),
            )
            .finish()
    }
}

// id is the SHA-256 hash of the ed25519 public key
fn gen_id(public_key: &PublicKey) -> [u8; ID_LENGTH] {
    let mut digest = [0u8; ID_LENGTH];
    sha::SHA256(public_key.as_ref(), &mut digest);
    digest
}
