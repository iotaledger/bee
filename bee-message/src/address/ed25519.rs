// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::ValidationError, signature::Ed25519Signature, util::hex_decode};

use bee_packable::packable::Packable;

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519::{PublicKey, Signature},
};

/// An Ed25519 address.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Ed25519Address([u8; Self::LENGTH]);

impl Ed25519Address {
    /// The [`Address`](crate::address::Address) kind of an [`Ed25519Address`].
    pub const KIND: u8 = 0;
    /// The length, in bytes, of an [`Ed25519Address`].
    pub const LENGTH: usize = 32;

    /// Creates a new [`Ed25519Address`].
    pub fn new(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }

    /// Verifies an [`Ed25519Signature`] for a message against the [`Ed25519Address`].
    pub fn verify(&self, signature: &Ed25519Signature, message: &[u8]) -> Result<(), ValidationError> {
        let address = Blake2b256::digest(signature.public_key());

        if self.0 != *address {
            return Err(ValidationError::SignaturePublicKeyMismatch {
                expected: hex::encode(self.0),
                actual: hex::encode(address),
            });
        }

        if !PublicKey::try_from_bytes(*signature.public_key())?
            .verify(&Signature::from_bytes(*signature.signature()), message)
        {
            return Err(ValidationError::InvalidSignature);
        }

        Ok(())
    }
}

impl From<[u8; Self::LENGTH]> for Ed25519Address {
    fn from(bytes: [u8; Self::LENGTH]) -> Self {
        Self::new(bytes)
    }
}

impl AsRef<[u8]> for Ed25519Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl core::ops::Deref for Ed25519Address {
    type Target = [u8; Self::LENGTH];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::str::FromStr for Ed25519Address {
    type Err = ValidationError;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        Ok(Ed25519Address::from(hex_decode(hex)?))
    }
}

impl core::fmt::Display for Ed25519Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        // Encodes to a base 16 hexadecimal string.
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for Ed25519Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Ed25519Address({})", self)
    }
}
