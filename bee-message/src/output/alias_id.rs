// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use core::{convert::TryInto, str::FromStr};

///
#[derive(Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd, derive_more::From)]
pub struct AliasId([u8; Self::LENGTH]);

impl AliasId {
    ///
    pub const LENGTH: usize = 20;

    /// Creates a new `AliasId`.
    pub fn new(bytes: [u8; Self::LENGTH]) -> Self {
        bytes.into()
    }
}

#[cfg(feature = "serde1")]
string_serde_impl!(AliasId);

impl FromStr for AliasId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; Self::LENGTH] = hex::decode(s)
            .map_err(|_| Self::Err::InvalidHexadecimalChar(s.to_owned()))?
            .try_into()
            .map_err(|_| Self::Err::InvalidHexadecimalLength(Self::LENGTH * 2, s.len()))?;

        Ok(AliasId::from(bytes))
    }
}

impl AsRef<[u8]> for AliasId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl core::fmt::Display for AliasId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for AliasId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "AliasId({})", self)
    }
}

impl Packable for AliasId {
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