// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::error::ValidationError;

use bee_packable::{PackError, Packable, Packer, UnpackError, Unpacker};

use alloc::{borrow::ToOwned, boxed::Box};
use core::{
    convert::{Infallible, TryInto},
    str::FromStr,
};

/// The number of bytes in a BLS address.
pub const BLS_ADDRESS_LENGTH: usize = 49;

/// A BLS address.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlsAddress(Box<[u8]>);

#[allow(clippy::len_without_is_empty)]
impl BlsAddress {
    /// The address kind of a BLS address.
    pub const KIND: u8 = 1;

    /// Creates a new BLS address.
    pub fn new(address: [u8; BLS_ADDRESS_LENGTH]) -> Self {
        address.into()
    }

    /// Returns the length of a BLS address.
    pub const fn len(&self) -> usize {
        BLS_ADDRESS_LENGTH
    }

    // TODO verification
}

impl Packable for BlsAddress {
    type PackError = Infallible;
    type UnpackError = Infallible;

    fn packed_len(&self) -> usize {
        BLS_ADDRESS_LENGTH
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        let bytes: [u8; BLS_ADDRESS_LENGTH] = self.0.to_vec().try_into().unwrap();

        bytes.pack(packer).map_err(PackError::infallible)
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let bytes = <[u8; BLS_ADDRESS_LENGTH]>::unpack(unpacker).map_err(UnpackError::infallible)?;

        Ok(Self::new(bytes))
    }
}

impl From<[u8; BLS_ADDRESS_LENGTH]> for BlsAddress {
    fn from(bytes: [u8; BLS_ADDRESS_LENGTH]) -> Self {
        Self(Box::new(bytes))
    }
}

impl FromStr for BlsAddress {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; BLS_ADDRESS_LENGTH] = hex::decode(s)
            .map_err(|_| Self::Err::InvalidHexadecimalChar(s.to_owned()))?
            .try_into()
            .map_err(|_| Self::Err::InvalidHexadecimalLength(BLS_ADDRESS_LENGTH * 2, s.len()))?;

        Ok(BlsAddress::from(bytes))
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
