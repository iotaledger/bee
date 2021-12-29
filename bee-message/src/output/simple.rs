// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, constant::IOTA_SUPPLY, Error};

use bee_packable::bounded::BoundedU64;

use core::ops::RangeInclusive;

pub(crate) type SimpleOutputAmount =
    BoundedU64<{ *SimpleOutput::AMOUNT_RANGE.start() }, { *SimpleOutput::AMOUNT_RANGE.end() }>;

/// Describes a simple output that can only hold IOTAs.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, bee_packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct SimpleOutput {
    // Deposit address of the output.
    address: Address,
    // Amount of IOTA tokens held by the output.
    #[packable(unpack_error_with = Error::InvalidAmount)]
    amount: SimpleOutputAmount,
}

impl SimpleOutput {
    /// The output kind of a [`SimpleOutput`].
    pub const KIND: u8 = 0;
    /// Valid amounts for a [`SimpleOutput`].
    pub const AMOUNT_RANGE: RangeInclusive<u64> = 1..=IOTA_SUPPLY;

    /// Creates a new [`SimpleOutput`].
    pub fn new(address: Address, amount: u64) -> Result<Self, Error> {
        amount
            .try_into()
            .map(|amount| Self { address, amount })
            .map_err(Error::InvalidAmount)
    }

    /// Returns the address of a [`SimpleOutput`].
    #[inline(always)]
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the amount of a [`SimpleOutput`].
    #[inline(always)]
    pub fn amount(&self) -> u64 {
        self.amount.get()
    }
}
