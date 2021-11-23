// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{constants::DUST_THRESHOLD, Error};

use bee_common::packable::{Packable, Read, Write};

use core::convert::TryFrom;

/// Defines the amount of IOTAs used as dust deposit that have to be returned to Sender.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct DustDepositReturnFeatureBlock {
    // Amount of IOTA coins the consuming transaction should deposit to the address defined in SenderFeatureBlock.
    amount: u64,
}

impl TryFrom<u64> for DustDepositReturnFeatureBlock {
    type Error = Error;

    fn try_from(amount: u64) -> Result<Self, Self::Error> {
        if amount < DUST_THRESHOLD {
            return Err(Error::InvalidDustDepositReturnFeatureBlock(amount));
        }

        Ok(Self { amount })
    }
}

impl DustDepositReturnFeatureBlock {
    /// The [`FeatureBlock`] kind of a [`DustDepositReturnFeatureBlock`].
    pub const KIND: u8 = 2;

    /// Creates a new [`DustDepositReturnFeatureBlock`].
    pub fn new(amount: u64) -> Result<Self, Error> {
        Self::try_from(amount)
    }

    /// Returns the amount.
    pub fn amount(&self) -> u64 {
        self.amount
    }
}

impl Packable for DustDepositReturnFeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.amount.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.amount.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Self::try_from(u64::unpack_inner::<R, CHECK>(reader)?)
    }
}
