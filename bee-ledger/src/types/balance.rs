// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{error::Error, BalanceDiff};

use bee_common::packable::{Packable, Read, Write};

use std::ops::Add;

/// Holds the balance of an address.
#[derive(Debug, Default)]
pub struct Balance {
    amount: u64,
    dust_allowance: u64,
    dust_output: u64,
}

impl Balance {
    /// Creates a new `Balance`.
    pub fn new(amount: u64, dust_allowance: u64, dust_output: u64) -> Self {
        Self {
            amount,
            dust_allowance,
            dust_output,
        }
    }

    /// Returns the amount of the `Balance`.
    pub fn amount(&self) -> u64 {
        self.amount
    }

    /// Returns the dust allowance of the `Balance`.
    pub fn dust_allowance(&self) -> u64 {
        self.dust_allowance
    }

    /// Returns the dust output of the `Balance`.
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

        // Given the nature of Utxo, this is never supposed to happen.
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

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let amount = u64::unpack_inner::<R, CHECK>(reader)?;
        let dust_allowance = u64::unpack_inner::<R, CHECK>(reader)?;
        let dust_output = u64::unpack_inner::<R, CHECK>(reader)?;

        Ok(Balance::new(amount, dust_allowance, dust_output))
    }
}
