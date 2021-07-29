// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::ValidationError, util::hex_decode};

use bee_packable::Packable;

/// A BLS address.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct BlsAddress(#[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))] [u8; Self::LENGTH]);

impl BlsAddress {
    /// The [`Address`](crate::address::Address) kind of a [`BlsAddress`].
    pub const KIND: u8 = 1;
    /// The length, in bytes, of a [`BlsAddress`].
    pub const LENGTH: usize = 49;

    /// Creates a new [`BlsAddress`].
    pub fn new(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }

    // TODO verification
    // pub fn verify(&self, signature: &BlsSignature, msg: &[u8]) -> Result<(), ValidationError> {}
}

impl From<[u8; Self::LENGTH]> for BlsAddress {
    fn from(bytes: [u8; Self::LENGTH]) -> Self {
        Self::new(bytes)
    }
}

impl AsRef<[u8]> for BlsAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl core::ops::Deref for BlsAddress {
    type Target = [u8; Self::LENGTH];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::str::FromStr for BlsAddress {
    type Err = ValidationError;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        Ok(BlsAddress::from(hex_decode(hex)?))
    }
}

impl core::fmt::Display for BlsAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(&self))
    }
}

impl core::fmt::Debug for BlsAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "BlsAddress({})", self)
    }
}
