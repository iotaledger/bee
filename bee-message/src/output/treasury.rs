// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{constant::IOTA_SUPPLY, Error};

use bee_common::packable::{Packable as OldPackable, Read, Write};
use bee_packable::bounded::BoundedU64;

use core::ops::RangeInclusive;

pub(crate) type TreasuryAmount =
    BoundedU64<{ *TreasuryOutput::AMOUNT_RANGE.start() }, { *TreasuryOutput::AMOUNT_RANGE.end() }>;

/// [`TreasuryOutput`] is an output which holds the treasury of a network.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, bee_packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = Error::InvalidTreasuryAmount)]
pub struct TreasuryOutput {
    amount: TreasuryAmount,
}

impl TreasuryOutput {
    /// The [`Output`](crate::output::Output) kind of a [`TreasuryOutput`].
    pub const KIND: u8 = 2;
    /// The allowed range of the amount of a [`TreasuryOutput`].
    pub const AMOUNT_RANGE: RangeInclusive<u64> = 0..=IOTA_SUPPLY;

    /// Creates a new [`TreasuryOutput`].
    pub fn new(amount: u64) -> Result<Self, Error> {
        amount
            .try_into()
            .map(|amount| Self { amount })
            .map_err(Error::InvalidTreasuryAmount)
    }

    /// Returns the amount of a [`TreasuryOutput`].
    #[inline(always)]
    pub fn amount(&self) -> u64 {
        self.amount.get()
    }
}

impl OldPackable for TreasuryOutput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.amount().packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.amount().pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Self::new(u64::unpack_inner::<R, CHECK>(reader)?)
    }
}
