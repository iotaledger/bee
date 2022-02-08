// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    constant::{DUST_DEPOSIT_MIN, IOTA_SUPPLY},
    Error,
};

use packable::{bounded::BoundedU64, Packable};

use std::ops::RangeInclusive;

/// Divisor used to compute the allowed dust outputs on an address.
pub const DUST_ALLOWANCE_DIVISOR: u64 = 100_000;
/// Maximum number of dust outputs for an address.
pub const DUST_OUTPUTS_MAX: u64 = 100;

#[cfg_attr(doc_cfg, doc(cfg(feature = "cpt2")))]
pub(crate) type DustAllowanceAmount = BoundedU64<
    { *SignatureLockedDustAllowanceOutput::AMOUNT_RANGE.start() },
    { *SignatureLockedDustAllowanceOutput::AMOUNT_RANGE.end() },
>;

/// The maximum number of allowed dust outputs on an address is `dust_allowance_sum` divided by `DUST_ALLOWANCE_DIVISOR`
/// and rounded down, i.e. 10 outputs for each 1 Mi deposited. `dust_allowance_sum` is the sum of all the amounts of all
/// unspent `SigLockedDustAllowanceOutputs` on this address. Regardless of `dust_allowance_sum`, the number of dust
/// outputs must never exceed `DUST_OUTPUTS_MAX` per address.
pub fn dust_outputs_max(dust_allowance_sum: u64) -> u64 {
    DUST_OUTPUTS_MAX.min(dust_allowance_sum / DUST_ALLOWANCE_DIVISOR)
}

/// A [`SignatureLockedDustAllowanceOutput`] functions like a `SignatureLockedSingleOutput` but as a special property it
/// is used to increase the allowance/amount of dust outputs on a given address.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
#[cfg_attr(doc_cfg, doc(cfg(feature = "cpt2")))]
pub struct SignatureLockedDustAllowanceOutput {
    address: Address,
    #[packable(unpack_error_with = Error::InvalidDustAllowanceAmount)]
    amount: DustAllowanceAmount,
}

impl SignatureLockedDustAllowanceOutput {
    /// The output kind of a [`SignatureLockedDustAllowanceOutput`].
    pub const KIND: u8 = 1;
    /// Valid amounts for a [`SignatureLockedDustAllowanceOutput`].
    pub const AMOUNT_RANGE: RangeInclusive<u64> = DUST_DEPOSIT_MIN..=IOTA_SUPPLY;

    /// Creates a new [`SignatureLockedDustAllowanceOutput`].
    pub fn new(address: Address, amount: u64) -> Result<Self, Error> {
        Ok(Self {
            address,
            amount: amount.try_into().map_err(Error::InvalidDustAllowanceAmount)?,
        })
    }

    /// Returns the address of a [`SignatureLockedDustAllowanceOutput`].
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the amount of a [`SignatureLockedDustAllowanceOutput`].
    pub fn amount(&self) -> u64 {
        self.amount.get()
    }
}
