// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constant::{DUST_DEPOSIT_MIN, IOTA_SUPPLY},
    Error,
};

use packable::bounded::BoundedU64;

use core::ops::RangeInclusive;

pub(crate) type DustDepositAmount = BoundedU64<
    { *DustDepositReturnUnlockCondition::AMOUNT_RANGE.start() },
    { *DustDepositReturnUnlockCondition::AMOUNT_RANGE.end() },
>;

/// Defines the amount of IOTAs used as dust deposit that have to be returned to the sender
/// [`Address`](crate::address::Address).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = Error::InvalidDustDepositAmount)]
pub struct DustDepositReturnUnlockCondition(
    // Amount of IOTA coins the consuming transaction should deposit to the [`Address`](crate::address::Address)
    // defined in [`SenderUnlockCondition`].
    DustDepositAmount,
);

impl TryFrom<u64> for DustDepositReturnUnlockCondition {
    type Error = Error;

    fn try_from(amount: u64) -> Result<Self, Self::Error> {
        amount.try_into().map(Self).map_err(Error::InvalidDustDepositAmount)
    }
}

impl DustDepositReturnUnlockCondition {
    /// The [`UnlockCondition`](crate::output::UnlockCondition) kind of a [`DustDepositReturnUnlockCondition`].
    pub const KIND: u8 = 2;
    /// Valid amounts for a [`DustDepositReturnUnlockCondition`].
    pub const AMOUNT_RANGE: RangeInclusive<u64> = DUST_DEPOSIT_MIN..=IOTA_SUPPLY;

    /// Creates a new [`DustDepositReturnUnlockCondition`].
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
