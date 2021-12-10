// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, constants::IOTA_SUPPLY, Error};

use bee_common::packable::{Packable, Read, Write};

use core::ops::RangeInclusive;

/// Describes a simple output that can only hold IOTAs.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct SimpleOutput {
    // Deposit address of the output.
    address: Address,
    // Amount of IOTA tokens held by the output.
    amount: u64,
}

impl SimpleOutput {
    /// The output kind of a [`SimpleOutput`].
    pub const KIND: u8 = 0;
    /// Valid amounts for a [`SimpleOutput`].
    pub const AMOUNT_RANGE: RangeInclusive<u64> = 1..=IOTA_SUPPLY;

    /// Creates a new [`SimpleOutput`].
    pub fn new(address: Address, amount: u64) -> Result<Self, Error> {
        validate_amount(amount)?;

        Ok(Self { address, amount })
    }

    /// Returns the address of a [`SimpleOutput`].
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the amount of a [`SimpleOutput`].
    pub fn amount(&self) -> u64 {
        self.amount
    }
}

impl Packable for SimpleOutput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.address.packed_len() + self.amount.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.address.pack(writer)?;
        self.amount.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let address = Address::unpack_inner::<R, CHECK>(reader)?;
        let amount = u64::unpack_inner::<R, CHECK>(reader)?;

        if CHECK {
            validate_amount(amount)?;
        }

        Ok(Self { address, amount })
    }
}

#[inline]
fn validate_amount(amount: u64) -> Result<(), Error> {
    if !SimpleOutput::AMOUNT_RANGE.contains(&amount) {
        return Err(Error::InvalidAmount(amount));
    }

    Ok(())
}
