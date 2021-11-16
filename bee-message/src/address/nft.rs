// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{output::NftId, Error};

use bee_common::packable::{Packable, Read, Write};

use core::{convert::TryInto, ops::Deref, str::FromStr};

/// A NFT address.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, derive_more::From)]
pub struct NftAddress(NftId);

#[allow(clippy::len_without_is_empty)]
impl NftAddress {
    /// The address kind of a NFT address.
    pub const KIND: u8 = 16;
    /// The length of a NFT address.
    pub const LENGTH: usize = 20;

    /// Creates a new NFT address.
    pub fn new(id: NftId) -> Self {
        id.into()
    }

    /// Returns the length of an NFT address.
    pub fn len(&self) -> usize {
        Self::LENGTH
    }

    ///
    pub fn id(&self) -> &NftId {
        &self.0
    }

    // /// Verifies a [`NftSignature`] for a message against the [`NftAddress`].
    // pub fn verify(&self, msg: &[u8], signature: &NftSignature) -> Result<(), Error> {
    //     Ok(())
    // }
}

#[cfg(feature = "serde1")]
string_serde_impl!(NftAddress);

impl FromStr for NftAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; Self::LENGTH] = hex::decode(s)
            .map_err(|_| Self::Err::InvalidHexadecimalChar(s.to_owned()))?
            .try_into()
            .map_err(|_| Self::Err::InvalidHexadecimalLength(Self::LENGTH * 2, s.len()))?;

        Ok(NftAddress::from(NftId::from(bytes)))
    }
}

impl Deref for NftAddress {
    type Target = NftId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for NftAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl core::fmt::Display for NftAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for NftAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "NftAddress({})", self)
    }
}

impl Packable for NftAddress {
    type Error = Error;

    fn packed_len(&self) -> usize {
        Self::LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new(NftId::unpack_inner::<R, CHECK>(reader)?))
    }
}
