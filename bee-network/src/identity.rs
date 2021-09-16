// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that deals with network identities.

use crate::consts::ID_LENGTH;

use crypto::{hashes::sha, signatures::ed25519};

use std::{
    fmt,
    sync::{Arc, RwLock},
};

/// A type that represents a local identity, which is able to sign messages.
#[derive(Clone)]
pub struct LocalIdentity {
    secret_key: Arc<RwLock<ed25519::SecretKey>>,
    identity: Identity,
}

impl LocalIdentity {
    /// Creates a new local identity.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a local identity from a 'base58' encoded ED25519 private key.
    pub fn from_bs58_secret_key_str(sk: &str) -> Self {
        let sk = bs58::decode(sk).into_vec().expect("error restoring secret key");
        if sk.len() != 32 {
            panic!("error restoring secret key");
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&sk[..32]);

        let secret_key = ed25519::SecretKey::from_bytes(bytes);

        let public_key = secret_key.public_key();
        let identity = Identity::from_public_key(public_key);

        Self {
            secret_key: Arc::new(RwLock::new(secret_key)),
            identity,
        }
    }

    /// Returns the public key of this identity.
    pub fn public_key(&self) -> ed25519::PublicKey {
        self.identity.public_key()
    }

    /// Returns the ID of this local identity.
    pub fn id_string(&self) -> String {
        self.identity.id_string()
    }

    /// Signs a message using the private key.
    pub fn sign(&self, msg: &[u8]) -> ed25519::Signature {
        self.secret_key.read().expect("error getting the lock").sign(msg)
    }
}

impl Default for LocalIdentity {
    fn default() -> Self {
        let secret_key = ed25519::SecretKey::generate().expect("error generating secret key");
        let identity = Identity::from_public_key(secret_key.public_key());

        Self {
            secret_key: Arc::new(RwLock::new(secret_key)),
            identity,
        }
    }
}

impl fmt::Debug for LocalIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalIdentity")
            .field("identity", &self.identity)
            .finish()
    }
}

/// A general identity of a peer.
#[derive(Clone)]
pub struct Identity {
    id: [u8; ID_LENGTH],
    public_key: Arc<RwLock<ed25519::PublicKey>>,
}

impl Identity {
    /// Creates an identity from an ED25519 public key.
    pub fn from_public_key(public_key: ed25519::PublicKey) -> Self {
        let id = gen_id(&public_key);
        Self {
            id,
            public_key: Arc::new(RwLock::new(public_key)),
        }
    }

    /// Returns the public key of this identity.
    pub fn public_key(&self) -> ed25519::PublicKey {
        let guard = self.public_key.read().expect("error getting the lock");
        let bytes = guard.as_ref();
        let mut pk = [0u8; 32];
        pk.copy_from_slice(bytes);
        ed25519::PublicKey::try_from_bytes(pk).expect("error cloning public key")
    }

    /// Returns the 'base58' string representation (created from of the first 8 bytes of the 32 byte long internal id)
    pub fn id_string(&self) -> String {
        bs58::encode(&self.id[..8]).into_string()
    }
}

impl fmt::Debug for Identity {
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
fn gen_id(public_key: &ed25519::PublicKey) -> [u8; ID_LENGTH] {
    let mut digest = [0u8; ID_LENGTH];
    sha::SHA256(public_key.as_ref(), &mut digest);
    digest
}
