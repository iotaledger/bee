// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, error::ValidationError, MessageUnpackError, IOTA_SUPPLY};

use bee_packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use core::ops::RangeInclusive;

/// Valid amounts for a signature locked single output.
pub const SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT: RangeInclusive<u64> = 1..=IOTA_SUPPLY;

/// An output type which can be unlocked via a signature. It deposits onto one single address.
///
/// A [`SignatureLockedSingleOutput`] must:
/// * Contain an amount <= [`IOTA_SUPPLY`].
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct SignatureLockedSingleOutput {
    address: Address,
    amount: u64,
}

impl SignatureLockedSingleOutput {
    /// The output kind of a [`SignatureLockedSingleOutput`].
    pub const KIND: u8 = 0;

    /// Creates a new [`SignatureLockedSingleOutput`].
    pub fn new(address: Address, amount: u64) -> Result<Self, ValidationError> {
        validate_amount(amount)?;

        Ok(Self { address, amount })
    }

    /// Returns the address of a [`SignatureLockedSingleOutput`].
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the amount of a [`SignatureLockedSingleOutput`].
    pub fn amount(&self) -> u64 {
        self.amount
    }
}

impl Packable for SignatureLockedSingleOutput {
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.address.packed_len() + self.amount.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.address.pack(packer)?;
        self.amount.pack(packer)
    }

    fn unpack<U: Unpacker, const CHECK: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let address = Address::unpack::<_, CHECK>(unpacker)?;

        let amount = u64::unpack::<_, CHECK>(unpacker).infallible()?;
        validate_amount(amount).map_err(UnpackError::from_packable)?;

        Ok(Self { address, amount })
    }
}

fn validate_amount(amount: u64) -> Result<(), ValidationError> {
    if !SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT.contains(&amount) {
        Err(ValidationError::InvalidAmount(amount))
    } else {
        Ok(())
    }
}
