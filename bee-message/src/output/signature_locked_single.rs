// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, error::ValidationError, MessageUnpackError, IOTA_SUPPLY};

use bee_packable::{bounded::BoundedU64, Packable};

use core::ops::RangeInclusive;

/// Valid amounts for a signature locked single output.
pub const SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT: RangeInclusive<u64> = 1..=IOTA_SUPPLY;

pub(crate) type Amount =
    BoundedU64<{ *SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT.start() }, { *SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT.end() }>;

/// An output type which can be unlocked via a signature. It deposits onto one single address.
///
/// A [`SignatureLockedSingleOutput`] must:
/// * Contain an amount <= [`IOTA_SUPPLY`].
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = MessageUnpackError)]
pub struct SignatureLockedSingleOutput {
    address: Address,
    #[packable(unpack_error_with = ValidationError::InvalidAmount)]
    amount: Amount,
}

impl SignatureLockedSingleOutput {
    /// The output kind of a [`SignatureLockedSingleOutput`].
    pub const KIND: u8 = 0;

    /// Creates a new [`SignatureLockedSingleOutput`].
    pub fn new(address: Address, amount: u64) -> Result<Self, ValidationError> {
        amount
            .try_into()
            .map(|amount| Self { address, amount })
            .map_err(ValidationError::InvalidAmount)
    }

    /// Returns the address of a [`SignatureLockedSingleOutput`].
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the amount of a [`SignatureLockedSingleOutput`].
    pub fn amount(&self) -> u64 {
        self.amount.get()
    }
}
