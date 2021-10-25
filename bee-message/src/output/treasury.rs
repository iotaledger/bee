// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{constants::IOTA_SUPPLY, Error};

use bee_packable::{
    bounded::BoundedU64,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use core::ops::RangeInclusive;
use std::convert::Infallible;

/// The allowed range of the amount of a `TreasuryOutput`.
pub const TREASURY_OUTPUT_AMOUNT: RangeInclusive<u64> = 0..=IOTA_SUPPLY;

/// `TreasuryOutput` is an output which holds the treasury of a network.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct TreasuryOutput {
    #[packable(unpack_error_with = Error::InvalidTreasuryAmount)]
    amount: BoundedU64<0, IOTA_SUPPLY>,
}

impl TreasuryOutput {
    /// The output kind of a `TreasuryOutput`.
    pub const KIND: u8 = 2;

    /// Creates a new `TreasuryOutput`.
    pub fn new(amount: u64) -> Result<Self, Error> {
        Ok(Self {
            amount: amount.try_into().map_err(Error::InvalidTreasuryAmount)?,
        })
    }

    /// Returns the amount of a `TreasuryOutput`.
    pub fn amount(&self) -> u64 {
        self.amount.get()
    }
}
