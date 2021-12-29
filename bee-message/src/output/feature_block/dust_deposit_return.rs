// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{constant::DUST_DEPOSIT_MIN, Error};

use bee_packable::bounded::BoundedU64;

pub(crate) type DustDepositAmount = BoundedU64<DUST_DEPOSIT_MIN, { u64::MAX }>;

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
