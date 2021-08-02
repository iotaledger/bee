// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_packable::Packable;

/// A BLS signature.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct BlsSignature(#[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))] [u8; Self::LENGTH]);

impl BlsSignature {
    /// The [`Signature`](crate::signature::Signature) kind of a [`BlsSignature`].
    pub const KIND: u8 = 1;
    /// Length, in bytes, of a [`BlsSignature`] .
    pub const LENGTH: usize = 64;

    /// Creates a new [`BlsSignature`].
    pub fn new(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }
}

impl From<[u8; Self::LENGTH]> for BlsSignature {
    fn from(bytes: [u8; Self::LENGTH]) -> Self {
        Self::new(bytes)
    }
}

impl AsRef<[u8]> for BlsSignature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl core::ops::Deref for BlsSignature {
    type Target = [u8; Self::LENGTH];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
