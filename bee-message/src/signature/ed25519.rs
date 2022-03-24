// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Ed25519Address, Error};

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519::{PublicKey, Signature, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH},
};

use core::ops::Deref;

/// An Ed25519 signature.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Ed25519Signature {
    public_key: [u8; Self::PUBLIC_KEY_LENGTH],
    #[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))]
    signature: [u8; Self::SIGNATURE_LENGTH],
}

impl Ed25519Signature {
    /// The signature kind of an [`Ed25519Signature`].
    pub const KIND: u8 = 0;
    /// Length of an ED25519 public key.
    pub const PUBLIC_KEY_LENGTH: usize = PUBLIC_KEY_LENGTH;
    /// Length of an ED25519 signature.
    pub const SIGNATURE_LENGTH: usize = SIGNATURE_LENGTH;

    /// Creates a new [`Ed25519Signature`].
    pub fn new(public_key: [u8; Self::PUBLIC_KEY_LENGTH], signature: [u8; Self::SIGNATURE_LENGTH]) -> Self {
        Self { public_key, signature }
    }

    /// Returns the public key of an [`Ed25519Signature`].
    pub fn public_key(&self) -> &[u8; Self::PUBLIC_KEY_LENGTH] {
        &self.public_key
    }

    /// Return the actual signature of an [`Ed25519Signature`].
    pub fn signature(&self) -> &[u8; Self::SIGNATURE_LENGTH] {
        &self.signature
    }

    /// Verifies the [`Ed25519Signature`] for a message against an [`Ed25519Address`].
    pub fn is_valid(&self, message: &[u8], address: &Ed25519Address) -> Result<(), Error> {
        let signature_address: [u8; PUBLIC_KEY_LENGTH] = Blake2b256::digest(&self.public_key).into();

        if address.deref() != &signature_address {
            return Err(Error::SignaturePublicKeyMismatch {
                expected: prefix_hex::encode(address.as_ref()),
                actual: prefix_hex::encode(signature_address),
            });
        }

        if !PublicKey::try_from_bytes(self.public_key)?.verify(&Signature::from_bytes(self.signature), message) {
            return Err(Error::InvalidSignature);
        }

        Ok(())
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    /// Defines an Ed25519 signature.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Ed25519SignatureDto {
        #[serde(rename = "type")]
        pub kind: u8,
        #[serde(rename = "publicKey")]
        pub public_key: String,
        pub signature: String,
    }
}
