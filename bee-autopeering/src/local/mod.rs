// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod salt;
pub mod service_map;

use salt::Salt;
use service_map::{ServiceMap, ServiceTransport};

use crate::{
    delay::DelayFactory,
    event::{Event, EventTx},
    hash,
    peer::{peer_id::PeerId, peerstore::PeerStore},
    task::{Repeat, ShutdownRx},
    time,
};

use crypto::signatures::ed25519::{PublicKey, SecretKey as PrivateKey, Signature, SECRET_KEY_LENGTH};

use std::{
    convert::TryInto,
    fmt,
    hash::{Hash, Hasher},
    iter,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};

/// A type that represents a local identity - able to sign outgoing messages.
#[derive(Clone, Default)]
pub struct Local {
    inner: Arc<RwLock<LocalInner>>,
}

pub struct LocalInner {
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

    /// Creates a local identity from a 'base16/hex' encoded ED25519 private key.
    pub fn from_bs16_encoded_private_key(private_key: impl AsRef<str>) -> Self {
        let mut private_key_bytes = [0u8; SECRET_KEY_LENGTH];
        hex::decode_to_slice(private_key.as_ref(), &mut private_key_bytes)
            .expect("error restoring private key from base16");

        Self::from_private_key_bytes(private_key_bytes)
    }

    /// Creates a local identity from a 'base58' encoded ED25519 private key.
    pub fn from_bs58_encoded_private_key(private_key: impl AsRef<str>) -> Self {
        // Restore the private key
        let mut private_key_bytes = [0u8; SECRET_KEY_LENGTH];
        bs58::decode(private_key.as_ref())
            .into(&mut private_key_bytes)
            .expect("error restoring private key from base58");

        Self::from_private_key_bytes(private_key_bytes)
    }

    /// Creates a local identity from bytes representing an ED25519 private key.
    pub fn from_private_key_bytes(private_key_bytes: [u8; SECRET_KEY_LENGTH]) -> Self {
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

    pub fn read(&self) -> RwLockReadGuard<LocalInner> {
        self.inner.read().expect("error getting read access")
    }

    pub fn write(&self) -> RwLockWriteGuard<LocalInner> {
        self.inner.write().expect("error getting write access")
    }
}

impl LocalInner {
    /// Returns the peer id of this identity.
    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    /// Returns the public key of this identity.
    pub fn public_key(&self) -> &PublicKey {
        self.peer_id().public_key()
    }

    /// Returns the current private salt of this identity.
    pub(crate) fn private_salt(&self) -> Option<&Salt> {
        self.private_salt.as_ref()
    }

    /// Sets a new private salt.
    pub(crate) fn set_private_salt(&mut self, salt: Salt) {
        self.private_salt.replace(salt);
    }

    /// Returns the current public salt of this identity.
    pub fn public_salt(&self) -> Option<&Salt> {
        self.public_salt.as_ref()
    }

    /// Sets a new public salt.
    pub(crate) fn set_public_salt(&mut self, salt: Salt) {
        self.public_salt.replace(salt);
    }

    /// Signs a message using the private key.
    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.private_key.sign(msg)
    }

    /// Adds a service to this local peer.
    pub fn add_service(&mut self, service_name: impl ToString, transport: ServiceTransport, port: u16) {
        self.services.insert(service_name, transport, port)
    }

    /// Returns the list of services this identity supports.
    pub(crate) fn services(&self) -> &ServiceMap {
        &self.services
    }
}

impl fmt::Debug for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Local")
            .field("peer_id", &self.read().peer_id())
            .finish()
    }
}

impl fmt::Display for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.read().peer_id())
    }
}

impl Eq for Local {}
impl PartialEq for Local {
    fn eq(&self, other: &Self) -> bool {
        self.read().peer_id() == other.read().peer_id()
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
