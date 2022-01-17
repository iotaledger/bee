// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{constant::IOTA_SUPPLY, Error};

use packable::bounded::BoundedU64;

use core::ops::RangeInclusive;

pub(crate) type TreasuryOutputAmount =
    BoundedU64<{ *TreasuryOutput::AMOUNT_RANGE.start() }, { *TreasuryOutput::AMOUNT_RANGE.end() }>;

/// [`TreasuryOutput`] is an output which holds the treasury of a network.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = Error::InvalidTreasuryOutputAmount)]
pub struct TreasuryOutput {
    amount: TreasuryOutputAmount,
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
            .map_err(Error::InvalidTreasuryOutputAmount)
    }

    /// Returns the amount of a [`TreasuryOutput`].
    #[inline(always)]
    pub fn amount(&self) -> u64 {
        self.amount.get()
    }
}
