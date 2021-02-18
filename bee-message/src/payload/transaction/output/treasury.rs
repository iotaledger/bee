// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{payload::transaction::constants::IOTA_SUPPLY, Error};

use bee_common::packable::{Packable, Read, Write};

use std::ops::RangeInclusive;

pub(crate) const TREASURY_OUTPUT_KIND: u8 = 2;
// TODO check if this is correct
const TREASURY_OUTPUT_AMOUNT: RangeInclusive<u64> = 1..=IOTA_SUPPLY;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TreasuryOutput {
    amount: u64,
}

impl TreasuryOutput {
    pub fn new(amount: u64) -> Result<Self, Error> {
        if !TREASURY_OUTPUT_AMOUNT.contains(&amount) {
            return Err(Error::InvalidTreasuryAmount(amount));
        }

        Ok(Self { amount })
    }

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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Self::new(u64::unpack(reader)?)
    }
}
