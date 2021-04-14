// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{constants::IOTA_SUPPLY, Error};

use bee_common::packable::{Packable, Read, Write};

use core::ops::RangeInclusive;

/// The allowed range of the amount of a `TreasuryOutput`.
pub const TREASURY_OUTPUT_AMOUNT: RangeInclusive<u64> = 0..=IOTA_SUPPLY;

/// `TreasuryOutput` is an output which holds the treasury of a network.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TreasuryOutput {
    amount: u64,
}

impl TreasuryOutput {
    /// The output kind of a `TreasuryOutput`.
    pub const KIND: u8 = 2;

    /// Creates a new `TreasuryOutput`.
    pub fn new(amount: u64) -> Result<Self, Error> {
        if !TREASURY_OUTPUT_AMOUNT.contains(&amount) {
            return Err(Error::InvalidTreasuryAmount(amount));
        }

        Ok(Self { amount })
    }

    /// Returns the amount of a `TreasuryOutput`.
    pub fn amount(&self) -> u64 {
        self.amount
    }
}

impl Packable for TreasuryOutput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.amount.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.amount.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Self::new(u64::unpack_inner::<R, CHECK>(reader)?)
    }
}
