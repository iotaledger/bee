// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod diff;

pub use diff::{BalanceDiff, BalanceDiffs};

use crate::model::Error;

use bee_common::packable::{Packable, Read, Write};

use std::ops::Add;

#[derive(Debug, Default)]
pub struct Balance {
    amount: u64,
    dust_allowance: u64,
    dust_output: u64,
}

impl Balance {
    pub fn new(amount: u64, dust_allowance: u64, dust_output: u64) -> Self {
        Self {
            amount,
            dust_allowance,
            dust_output,
        }
    }

    pub fn amount(&self) -> u64 {
        self.amount
    }

    pub fn dust_allowance(&self) -> u64 {
        self.dust_allowance
    }

    pub fn dust_output(&self) -> u64 {
        self.dust_output
    }
}

impl Add<&BalanceDiff> for Balance {
    type Output = Self;

    fn add(self, other: &BalanceDiff) -> Self {
        let amount = self.amount as i64 + other.amount();
        let dust_allowance = self.dust_allowance() as i64 + other.dust_allowance();
        let dust_output = self.dust_output as i64 + other.dust_output();

        // Given the nature of UTXO, this is never supposed to happen.
        assert!(amount >= 0);
        assert!(dust_allowance >= 0);
        assert!(dust_output >= 0);

        Self {
            amount: amount as u64,
            dust_allowance: dust_allowance as u64,
            dust_output: dust_output as u64,
        }
    }
}

impl Packable for Balance {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.amount.packed_len() + self.dust_allowance.packed_len() + self.dust_output.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.amount.pack(writer)?;
        self.dust_allowance.pack(writer)?;
        self.dust_output.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Balance::new(
            u64::unpack(reader)?,
            u64::unpack(reader)?,
            u64::unpack(reader)?,
        ))
    }
}
