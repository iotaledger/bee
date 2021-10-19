// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use core::{convert::TryInto, str::FromStr};

/// A BLS address.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, derive_more::From)]
pub struct BlsAddress([u8; Self::LENGTH]);

#[allow(clippy::len_without_is_empty)]
impl BlsAddress {
    /// The address kind of a BLS address.
    pub const KIND: u8 = 1;
    /// The length of a BLS address.
    pub const LENGTH: usize = 32;

    /// Creates a new BLS address.
    pub fn new(address: [u8; Self::LENGTH]) -> Self {
        address.into()
    }

    /// Returns the length of an BLS address.
    pub fn len(&self) -> usize {
        Self::LENGTH
    }

    // /// Verifies a [`BlsSignature`] for a message against the [`BlsAddress`].
    // pub fn verify(&self, msg: &[u8], signature: &BlsSignature) -> Result<(), Error> {
    //     Ok(())
    // }
}

#[cfg(feature = "serde1")]
string_serde_impl!(BlsAddress);

impl FromStr for BlsAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; Self::LENGTH] = hex::decode(s)
            .map_err(|_| Self::Err::InvalidHexadecimalChar(s.to_owned()))?
            .try_into()
            .map_err(|_| Self::Err::InvalidHexadecimalLength(Self::LENGTH * 2, s.len()))?;

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
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for BlsAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "BlsAddress({})", self)
    }
}

impl Packable for BlsAddress {
    type Error = Error;

    fn packed_len(&self) -> usize {
        Self::LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new(<[u8; Self::LENGTH]>::unpack_inner::<R, CHECK>(reader)?))
    }
}
