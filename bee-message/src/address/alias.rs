// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use core::{convert::TryInto, str::FromStr};

/// An alias address.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, derive_more::From)]
pub struct AliasAddress([u8; Self::LENGTH]);

#[allow(clippy::len_without_is_empty)]
impl AliasAddress {
    /// The address kind of an alias address.
    pub const KIND: u8 = 8;
    /// The length of an alias address.
    pub const LENGTH: usize = 20;

    /// Creates a new alias address.
    pub fn new(address: [u8; Self::LENGTH]) -> Self {
        address.into()
    }

    /// Returns the length of an alias address.
    pub fn len(&self) -> usize {
        Self::LENGTH
    }

    // /// Verifies a [`AliasSignature`] for a message against the [`AliasAddress`].
    // pub fn verify(&self, msg: &[u8], signature: &AliasSignature) -> Result<(), Error> {
    //     Ok(())
    // }
}

#[cfg(feature = "serde1")]
string_serde_impl!(AliasAddress);

impl FromStr for AliasAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; Self::LENGTH] = hex::decode(s)
            .map_err(|_| Self::Err::InvalidHexadecimalChar(s.to_owned()))?
            .try_into()
            .map_err(|_| Self::Err::InvalidHexadecimalLength(Self::LENGTH * 2, s.len()))?;

        Ok(AliasAddress::from(bytes))
    }
}

impl AsRef<[u8]> for AliasAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl core::fmt::Display for AliasAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for AliasAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "AliasAddress({})", self)
    }
}

impl Packable for AliasAddress {
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
