// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{constants::DUST_THRESHOLD, Error};

use bee_common::packable::{Packable, Read, Write};

use core::convert::TryFrom;

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ReturnAmountFeatureBlock(u64);

impl TryFrom<u64> for ReturnAmountFeatureBlock {
    type Error = Error;

    fn try_from(amount: u64) -> Result<Self, Self::Error> {
        if amount < DUST_THRESHOLD {
            return Err(Error::InvalidReturnAmountFeatureBlock(amount));
        }

        Ok(Self(amount))
    }
}

impl ReturnAmountFeatureBlock {
    /// The feature block kind of a `ReturnAmountFeatureBlock`.
    pub const KIND: u8 = 2;

    /// Creates a new `ReturnAmountFeatureBlock`.
    pub fn new(amount: u64) -> Result<Self, Error> {
        Self::try_from(amount)
    }

    /// Returns the return amount.
    pub fn amount(&self) -> u64 {
        self.0
    }
}

impl Packable for ReturnAmountFeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Self::try_from(u64::unpack_inner::<R, CHECK>(reader)?)
    }
}
