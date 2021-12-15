// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{output::NftId, util::hex_decode, Error};

use bee_common::packable::{Packable, Read, Write};

use derive_more::{AsRef, Deref, From};

use core::str::FromStr;

/// A NFT address.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, From, AsRef, Deref)]
#[as_ref(forward)]
pub struct NftAddress(NftId);

#[allow(clippy::len_without_is_empty)]
impl NftAddress {
    /// The [`Address`](crate::address::Address) kind of a NFT address.
    pub const KIND: u8 = 16;
    /// The length of a [`NftAddress`].
    pub const LENGTH: usize = 20;

    /// Creates a new [`NftAddress`].
    #[inline(always)]
    pub fn new(id: NftId) -> Self {
        Self::from(id)
    }

    /// Returns the [`NftId`] of an [`NftAddress`].
    #[inline(always)]
    pub fn id(&self) -> &NftId {
        &self.0
    }
}

#[cfg(feature = "serde1")]
string_serde_impl!(NftAddress);

impl FromStr for NftAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(NftAddress::from(NftId::from(hex_decode(s)?)))
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
