// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::ValidationError, signature::Ed25519Signature, util::hex_decode};

use bee_packable::packable::Packable;

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519::{PublicKey, Signature},
};

use core::str::FromStr;

/// The number of bytes in an Ed25519 address.
/// See <https://en.wikipedia.org/wiki/EdDSA#Ed25519> for more information.
pub const ED25519_ADDRESS_LENGTH: usize = 32;

/// An Ed25519 address.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Ed25519Address([u8; ED25519_ADDRESS_LENGTH]);

#[allow(clippy::len_without_is_empty)]
impl Ed25519Address {
    /// The address kind of an Ed25519 address.
    pub const KIND: u8 = 0;

    /// Creates a new Ed25519 address.
    pub fn new(address: [u8; ED25519_ADDRESS_LENGTH]) -> Self {
        address.into()
    }

    /// Returns the length of an Ed25519 address.
    pub const fn len(&self) -> usize {
        ED25519_ADDRESS_LENGTH
    }

    /// Verifies a [`Ed25519Signature`] for a message against the [`Ed25519Address`].
    pub fn verify(&self, msg: &[u8], signature: &Ed25519Signature) -> Result<(), ValidationError> {
        let address = Blake2b256::digest(signature.public_key());

        if self.0 != *address {
            return Err(ValidationError::SignaturePublicKeyMismatch(
                hex::encode(self.0),
                hex::encode(address),
            ));
        }

        if !PublicKey::from_compressed_bytes(*signature.public_key())?
            .verify(&Signature::from_bytes(*signature.signature()), msg)
        {
            return Err(ValidationError::InvalidSignature);
        }

        Ok(())
    }
}

impl From<[u8; ED25519_ADDRESS_LENGTH]> for Ed25519Address {
    fn from(bytes: [u8; ED25519_ADDRESS_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl FromStr for Ed25519Address {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Ed25519Address::from(hex_decode(s)?))
    }
}

impl AsRef<[u8]> for Ed25519Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl core::fmt::Display for Ed25519Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for Ed25519Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Ed25519Address({})", self)
    }
}
