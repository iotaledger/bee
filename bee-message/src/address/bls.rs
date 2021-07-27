// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::ValidationError, util::hex_decode};

use bee_packable::Packable;

use core::str::FromStr;

/// A BLS address.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct BlsAddress(#[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))] [u8; Self::LENGTH]);

#[allow(clippy::len_without_is_empty)]
impl BlsAddress {
    /// The address kind of a [`BlsAddress`].
    pub const KIND: u8 = 1;

    /// The length (in bytes) of a [`BlsAddress`].
    pub const LENGTH: usize = 49;

    /// Creates a new BLS address.
    pub fn new(address: [u8; Self::LENGTH]) -> Self {
        address.into()
    }

    // TODO verification
}

impl From<[u8; Self::LENGTH]> for BlsAddress {
    fn from(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }
}

impl FromStr for BlsAddress {
    type Err = ValidationError;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        Ok(BlsAddress::from(hex_decode(hex)?))
    }
}

impl AsRef<[u8]> for BlsAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0
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
