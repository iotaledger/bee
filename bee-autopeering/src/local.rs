// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{delay::Repeat, hash, identity::PeerId, peerstore::PeerStore, salt::Salt, ServiceMap};

use crypto::signatures::ed25519::{PublicKey, SecretKey as PrivateKey, Signature, SECRET_KEY_LENGTH};

use std::{
    convert::TryInto,
    fmt,
    hash::{Hash, Hasher},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

/// A type that represents a local identity, which is able to sign messages.
#[derive(Clone, Default)]
pub struct Local {
    inner: Arc<RwLock<LocalInner>>,
}

struct LocalInner {
    peer_id: PeerId,
    public_salt: Option<Salt>,
    private_key: PrivateKey,
    private_salt: Option<Salt>,
    services: ServiceMap,
}

impl Local {
    /// Creates a new local identity.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a local identity from a 'base58' encoded ED25519 private key.
    pub fn from_bs58_encoded_private_key(private_key: impl AsRef<str>) -> Self {
        // Restore the private key
        let mut private_key_bytes = [0u8; SECRET_KEY_LENGTH];
        bs58::decode(private_key.as_ref())
            .into(&mut private_key_bytes)
            .expect("error restoring private key");

        let private_key = PrivateKey::from_bytes(private_key_bytes);
        let public_key = private_key.public_key();
        let peer_id = PeerId::from_public_key(public_key);

        Self {
            inner: Arc::new(RwLock::new(LocalInner {
                peer_id,
                private_key,
                private_salt: None,
                public_salt: None,
                services: ServiceMap::default(),
            })),
        }
    }

    /// Returns the peer id of this identity.
    pub fn peer_id(&self) -> PeerId {
        self.read_inner().peer_id.clone()
    }

    /// Returns the public key of this identity.
    pub fn public_key(&self) -> PublicKey {
        self.read_inner().peer_id.public_key()
    }

    /// Returns the current private salt of this identity.
    pub(crate) fn private_salt(&self) -> Option<Salt> {
        self.read_inner().private_salt.clone()
    }

    /// Sets a new private salt.
    pub(crate) fn set_private_salt(&self, salt: Salt) {
        self.write_inner().private_salt.replace(salt);
    }

    /// Returns the current public salt of this identity.
    pub fn public_salt(&self) -> Option<Salt> {
        self.read_inner().public_salt.clone()
    }

    /// Sets a new public salt.
    pub(crate) fn set_public_salt(&self, salt: Salt) {
        self.write_inner().public_salt.replace(salt);
    }

    /// Signs a message using the private key.
    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.read_inner().private_key.sign(msg)
    }

    /// Returns the list of services this identity supports.
    pub fn services(&self) -> ServiceMap {
        self.read_inner().services.clone()
    }

    fn read_inner(&self) -> RwLockReadGuard<LocalInner> {
        self.inner.read().expect("error getting read access")
    }

    fn write_inner(&self) -> RwLockWriteGuard<LocalInner> {
        self.inner.write().expect("error getting write access")
    }
}

impl fmt::Debug for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Local").field("peer_id", &self.peer_id()).finish()
    }
}

impl fmt::Display for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.peer_id())
    }
}

impl Eq for Local {}
impl PartialEq for Local {
    fn eq(&self, other: &Self) -> bool {
        self.peer_id() == other.peer_id()
    }
}

impl Default for LocalInner {
    fn default() -> Self {
        let private_key = PrivateKey::generate().expect("error generating private key");
        let peer_id = PeerId::from_public_key(private_key.public_key());

        Self {
            peer_id,
            public_salt: Some(Salt::default()),
            private_key,
            private_salt: Some(Salt::default()),
            services: ServiceMap::default(),
        }
    }
}

impl Eq for LocalInner {}
impl PartialEq for LocalInner {
    fn eq(&self, other: &Self) -> bool {
        self.peer_id == other.peer_id
    }
}