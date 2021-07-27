// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_packable::Packable;

/// Length (in bytes) of an Ed25519 public key.
pub const ED25519_PUBLIC_KEY_LENGTH: usize = 32;

/// Length (in bytes) of an Ed26618 signature.
pub const ED25519_SIGNATURE_LENGTH: usize = 64;

/// An Ed25519 signature.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Ed25519Signature {
    public_key: [u8; ED25519_PUBLIC_KEY_LENGTH],
    #[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))]
    signature: [u8; ED25519_SIGNATURE_LENGTH],
}

impl Ed25519Signature {
    /// The signature kind of an [`Ed25519Signature`].
    pub const KIND: u8 = 0;

    /// Creates a new [`Ed25519Signature`].
    pub fn new(public_key: [u8; ED25519_PUBLIC_KEY_LENGTH], signature: [u8; ED25519_SIGNATURE_LENGTH]) -> Self {
        Self { public_key, signature }
    }

    /// Returns the public key of an [`Ed25519Signature`].
    pub fn public_key(&self) -> &[u8; ED25519_PUBLIC_KEY_LENGTH] {
        &self.public_key
    }

    /// Returns the actual signature of an [`Ed25519Signature`].
    pub fn signature(&self) -> &[u8; ED25519_SIGNATURE_LENGTH] {
        &self.signature
    }
}
