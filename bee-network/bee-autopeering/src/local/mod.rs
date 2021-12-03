// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod salt;
pub mod services;

use self::{
    salt::{Salt, SALT_LIFETIME_SECS},
    services::{ServiceMap, ServiceProtocol},
};

use crate::peer::PeerId;

use crypto::signatures::ed25519::{PublicKey, SecretKey as PrivateKey, Signature, SECRET_KEY_LENGTH};
use libp2p_core::identity::ed25519::Keypair;

use std::{
    fmt,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not create Salt from ED25519 keypair")]
    SaltFromEd25519Keypair,
    #[error("could not create Salt from Base16/Hex private key")]
    SaltFromBase16EncodedPrivateKey,
    #[error("could not create Salt from Base58 private key")]
    SaltFromBase58EncodedPrivateKey,
    #[error("could not deserialize Salt from Protobuf")]
    DeserializeFromProtobuf,
}

/// Represents a local entity.
///
/// It allows:
/// * message signing and verification;
/// * neighbor distance calculation;
/// * service announcements;
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

    /// Creates a local identity from an ED25519 keypair.
    pub fn from_keypair(keypair: Keypair) -> Result<Self, Error> {
        let private_key_bytes: [u8; SECRET_KEY_LENGTH] = keypair
            .secret()
            .as_ref()
            .try_into()
            .map_err(|_| Error::SaltFromEd25519Keypair)
            .expect("error restoring private key from ed25519 keypair");

        Ok(Self::from_private_key_bytes(private_key_bytes))
    }

    /// Creates a local identity from a 'base16/hex' encoded ED25519 private key.
    pub fn from_bs16_encoded_private_key(private_key: impl AsRef<str>) -> Result<Self, Error> {
        let mut private_key_bytes = [0u8; SECRET_KEY_LENGTH];
        hex::decode_to_slice(private_key.as_ref(), &mut private_key_bytes)
            .map_err(|_| Error::SaltFromBase16EncodedPrivateKey)?;

        Ok(Self::from_private_key_bytes(private_key_bytes))
    }

    /// Creates a local identity from a 'base58' encoded ED25519 private key.
    pub fn from_bs58_encoded_private_key(private_key: impl AsRef<str>) -> Result<Self, Error> {
        // Restore the private key
        let mut private_key_bytes = [0u8; SECRET_KEY_LENGTH];
        bs58::decode(private_key.as_ref())
            .into(&mut private_key_bytes)
            .map_err(|_| Error::SaltFromBase58EncodedPrivateKey)?;

        Ok(Self::from_private_key_bytes(private_key_bytes))
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
                private_salt: Some(Salt::new(SALT_LIFETIME_SECS)),
                public_salt: Some(Salt::new(SALT_LIFETIME_SECS)),
                services: ServiceMap::default(),
            })),
        }
    }

    /// Returns the peer id of this identity.
    pub(crate) fn peer_id(&self) -> PeerId {
        *self.read().peer_id()
    }

    /// Returns the public key of this identity.
    pub(crate) fn public_key(&self) -> PublicKey {
        *self.read().public_key()
    }

    /// Returns the current private salt of this identity.
    pub(crate) fn private_salt(&self) -> Option<Salt> {
        self.read().private_salt().cloned()
    }

    /// Sets a new private salt.
    pub(crate) fn set_private_salt(&self, salt: Salt) {
        self.write().set_private_salt(salt);
    }

    /// Returns the current public salt of this identity.
    pub(crate) fn public_salt(&self) -> Option<Salt> {
        self.read().public_salt().cloned()
    }

    /// Sets a new public salt.
    pub(crate) fn set_public_salt(&self, salt: Salt) {
        self.write().set_public_salt(salt);
    }

    /// Signs a message using the private key.
    pub(crate) fn sign(&self, msg: &[u8]) -> Signature {
        self.read().sign(msg)
    }

    /// Adds a service to this local peer.
    pub fn add_service(&self, service_name: impl ToString, protocol: ServiceProtocol, port: u16) {
        self.write().add_service(service_name, protocol, port);
    }

    /// Returns the list of services this identity supports.
    pub(crate) fn services(&self) -> ServiceMap {
        self.read().services().clone()
    }

    fn read(&self) -> RwLockReadGuard<LocalInner> {
        self.inner.read().expect("error getting read access")
    }

    fn write(&self) -> RwLockWriteGuard<LocalInner> {
        self.inner.write().expect("error getting write access")
    }
}

impl LocalInner {
    fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    fn public_key(&self) -> &PublicKey {
        self.peer_id().public_key()
    }

    fn private_salt(&self) -> Option<&Salt> {
        self.private_salt.as_ref()
    }

    fn set_private_salt(&mut self, salt: Salt) {
        self.private_salt.replace(salt);
    }

    fn public_salt(&self) -> Option<&Salt> {
        self.public_salt.as_ref()
    }

    fn set_public_salt(&mut self, salt: Salt) {
        self.public_salt.replace(salt);
    }

    fn sign(&self, msg: &[u8]) -> Signature {
        self.private_key.sign(msg)
    }

    fn add_service(&mut self, service_name: impl ToString, protocol: ServiceProtocol, port: u16) {
        self.services.insert(service_name, protocol, port)
    }

    fn services(&self) -> &ServiceMap {
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
