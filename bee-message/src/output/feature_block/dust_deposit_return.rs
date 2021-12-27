// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constant::{DUST_DEPOSIT_MIN, IOTA_SUPPLY},
    Error,
};

use bee_common::packable::{Packable as OldPackable, Read, Write};
use bee_packable::bounded::BoundedU64;

pub(crate) type DustDepositAmount = BoundedU64<DUST_DEPOSIT_MIN, { u64::MAX }>;

use core::ops::RangeInclusive;

/// Defines the amount of IOTAs used as dust deposit that have to be returned to the sender
/// [`Address`](crate::address::Address).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, bee_packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = Error::InvalidDustDepositAmount)]
pub struct DustDepositReturnFeatureBlock(
    // Amount of IOTA coins the consuming transaction should deposit to the [`Address`](crate::address::Address)
    // defined in [`SenderFeatureBlock`].
    DustDepositAmount,
);

impl TryFrom<u64> for DustDepositReturnFeatureBlock {
    type Error = Error;

    fn try_from(amount: u64) -> Result<Self, Self::Error> {
        amount.try_into().map(Self).map_err(Error::InvalidDustDepositAmount)
    }
}

impl DustDepositReturnFeatureBlock {
    /// The [`FeatureBlock`](crate::output::FeatureBlock) kind of a [`DustDepositReturnFeatureBlock`].
    pub const KIND: u8 = 2;
    /// Valid amounts for a [`DustDepositReturnFeatureBlock`].
    pub const AMOUNT_RANGE: RangeInclusive<u64> = DUST_DEPOSIT_MIN..=IOTA_SUPPLY;

    /// Creates a new [`DustDepositReturnFeatureBlock`].
    #[inline(always)]
    pub fn new(amount: u64) -> Result<Self, Error> {
        Self::try_from(amount)
    }

    /// Returns the amount.
    #[inline(always)]
    pub fn amount(&self) -> u64 {
        self.0.get()
    }
}

impl OldPackable for DustDepositReturnFeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.amount().packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.amount().pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let amount = u64::unpack_inner::<R, CHECK>(reader)?;

        if CHECK {
            validate_amount(amount)?;
        }

        Self::new(amount)
    }
}

#[inline]
fn validate_amount(amount: u64) -> Result<(), Error> {
    DustDepositReturnFeatureBlock::try_from(amount)?;

    Ok(())
}
