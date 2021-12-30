// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable as OldPackable, Read, Write};

use derive_more::From;

/// Defines a unix time until which the output can not be unlocked.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TimelockUnixFeatureBlock(
    // Unix time, seconds since unix epoch, starting from which the output can be consumed.
    u32,
);

impl TimelockUnixFeatureBlock {
    /// The [`FeatureBlock`](crate::output::FeatureBlock) kind of a [`TimelockUnixFeatureBlock`].
    pub const KIND: u8 = 4;

    /// Creates a new [`TimelockUnixFeatureBlock`].
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

impl OldPackable for TimelockUnixFeatureBlock {
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
