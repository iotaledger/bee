// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, constants::IOTA_SUPPLY, Error};

use bee_packable::{
    bounded::BoundedU64,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use core::ops::RangeInclusive;
use std::convert::Infallible;

/// Valid amounts for a signature locked single output.
pub const SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT: RangeInclusive<u64> = 1..=IOTA_SUPPLY;

/// An output type which can be unlocked via a signature. It deposits onto one single address.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct SignatureLockedSingleOutput {
    address: Address,
    #[packable(unpack_error_with = Error::InvalidAmount)]
    amount: BoundedU64<1, IOTA_SUPPLY>,
}

impl SignatureLockedSingleOutput {
    /// The output kind of a `SignatureLockedSingleOutput`.
    pub const KIND: u8 = 0;

    /// Creates a new `SignatureLockedSingleOutput`.
    pub fn new(address: Address, amount: u64) -> Result<Self, Error> {
        Ok(Self {
            address,
            amount: amount.try_into().map_err(Error::InvalidAmount)?,
        })
    }

    /// Returns the address of a `SignatureLockedSingleOutput`.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the amount of a `SignatureLockedSingleOutput`.
    pub fn amount(&self) -> u64 {
        self.amount.get()
    }
}
