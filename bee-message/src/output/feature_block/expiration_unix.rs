// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

/// Defines a unix time until which only the deposit [`Address`] is allowed to unlock the output.
/// After the expiration time, only the [`Address`] defined in the [`SenderFeatureBlock`] can unlock it.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, derive_more::From)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpirationUnixFeatureBlock {
    // Before this unix time, seconds since unix epoch, [`Address`] is allowed to unlock the output.
    // After that, only the [`Address`] defined in the [`SenderFeatureBlock`] can.
    timestamp: u32,
}

impl ExpirationUnixFeatureBlock {
    /// The [`FeatureBlock`] kind of an [`ExpirationUnixFeatureBlock`].
    pub const KIND: u8 = 6;

    /// Creates a new [`ExpirationUnixFeatureBlock`].
    pub fn new(timestamp: u32) -> Self {
        timestamp.into()
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }
}

impl Packable for ExpirationUnixFeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.timestamp.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.timestamp.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self {
            timestamp: u32::unpack_inner::<R, CHECK>(reader)?,
        })
    }
}
