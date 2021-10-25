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

/// Amount of tokens below which an output is considered a dust output.
pub const DUST_THRESHOLD: u64 = 1_000_000;
/// Divisor used to compute the allowed dust outputs on an address.
pub const DUST_ALLOWANCE_DIVISOR: u64 = 100_000;
/// Maximum number of dust outputs for an address.
pub const DUST_OUTPUTS_MAX: u64 = 100;
/// Valid amounts for a signature locked dust allowance output.
pub const SIGNATURE_LOCKED_DUST_ALLOWANCE_OUTPUT_AMOUNT: RangeInclusive<u64> = DUST_THRESHOLD..=IOTA_SUPPLY;

/// The maximum number of allowed dust outputs on an address is `dust_allowance_sum` divided by `DUST_ALLOWANCE_DIVISOR`
/// and rounded down, i.e. 10 outputs for each 1 Mi deposited. `dust_allowance_sum` is the sum of all the amounts of all
/// unspent `SigLockedDustAllowanceOutputs` on this address. Regardless of `dust_allowance_sum`, the number of dust
/// outputs must never exceed `DUST_OUTPUTS_MAX` per address.
pub fn dust_outputs_max(dust_allowance_sum: u64) -> u64 {
    DUST_OUTPUTS_MAX.min(dust_allowance_sum / DUST_ALLOWANCE_DIVISOR)
}

/// A `SignatureLockedDustAllowanceOutput` functions like a `SignatureLockedSingleOutput` but as a special property it
/// is used to increase the allowance/amount of dust outputs on a given address.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct SignatureLockedDustAllowanceOutput {
    address: Address,
    #[packable(unpack_error_with = Error::InvalidDustAllowanceAmount)]
    amount: BoundedU64<DUST_THRESHOLD, IOTA_SUPPLY>,
}

impl SignatureLockedDustAllowanceOutput {
    /// The output kind of a `SignatureLockedDustAllowanceOutput`.
    pub const KIND: u8 = 1;

    /// Creates a new `SignatureLockedDustAllowanceOutput`.
    pub fn new(address: Address, amount: u64) -> Result<Self, Error> {
        Ok(Self {
            address,
            amount: amount.try_into().map_err(Error::InvalidDustAllowanceAmount)?,
        })
    }

    /// Returns the address of a `SignatureLockedDustAllowanceOutput`.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the amount of a `SignatureLockedDustAllowanceOutput`.
    pub fn amount(&self) -> u64 {
        self.amount.get()
    }
}
