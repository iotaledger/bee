// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, derive_more::From)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpirationUnixFeatureBlock(u64);

impl ExpirationUnixFeatureBlock {
    /// The feature block kind of an `ExpirationUnixFeatureBlock`.
    pub const KIND: u8 = 6;

    /// Creates a new `ExpirationUnixFeatureBlock`.
    pub fn new(timestamp: u64) -> Self {
        timestamp.into()
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> u64 {
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
        Ok(Self::new(u64::unpack_inner::<R, CHECK>(reader)?))
    }
}
