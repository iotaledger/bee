// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, Error};

use bee_common::packable::{Packable, Read, Write};

use derive_more::From;

/// Identifies the validated sender of an output.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct SenderFeatureBlock(Address);

impl SenderFeatureBlock {
    /// The [`FeatureBlock`](crate::output::FeatureBlock) kind of a [`SenderFeatureBlock`].
    pub const KIND: u8 = 0;

    /// Creates a new [`SenderFeatureBlock`].
    #[inline(always)]
    pub fn new(address: Address) -> Self {
        Self(address)
    }

    /// Returns the sender [`Address`].
    #[inline(always)]
    pub fn address(&self) -> &Address {
        &self.0
    }
}

impl Packable for SenderFeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self(Address::unpack_inner::<R, CHECK>(reader)?))
    }
}
