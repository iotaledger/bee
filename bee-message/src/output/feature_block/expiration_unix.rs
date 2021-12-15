// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use derive_more::From;

/// Defines a unix time until which only the deposit [`Address`](crate::address::Address) is allowed to unlock the
/// output. After the expiration time, only the [`Address`](crate::address::Address) defined in the
/// [`SenderFeatureBlock`](crate::output::feature_block::SenderFeatureBlock) can unlock it.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpirationUnixFeatureBlock(
    // Before this unix time, seconds since unix epoch, [`Address`](crate::address::Address) is allowed to unlock the
    // output. After that, only the [`Address`](crate::address::Address) defined in the
    // [`SenderFeatureBlock`](crate::output::feature_block::SenderFeatureBlock) can.
    u32,
);

impl ExpirationUnixFeatureBlock {
    /// The [`FeatureBlock`](crate::output::FeatureBlock) kind of an [`ExpirationUnixFeatureBlock`].
    pub const KIND: u8 = 6;

    /// Creates a new [`ExpirationUnixFeatureBlock`].
    #[inline(always)]
    pub fn new(timestamp: u32) -> Self {
        Self(timestamp)
    }

    /// Returns the timestamp.
    #[inline(always)]
    pub fn timestamp(&self) -> u32 {
        self.0
    }
}

impl Packable for ExpirationUnixFeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self(u32::unpack_inner::<R, CHECK>(reader)?))
    }
}
