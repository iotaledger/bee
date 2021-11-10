// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{error::Error, BalanceDiff};

use bee_common::packable::{Packable, Read, Write};
use bee_message::constants::IOTA_SUPPLY;

/// Holds the balance of an address.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Balance {
    amount: u64,
}

impl Balance {
    /// Creates a new `Balance`.
    pub fn new(amount: u64) -> Result<Self, Error> {
        if amount > IOTA_SUPPLY {
            return Err(Error::InvalidBalance(amount));
        }

        Ok(Self { amount })
    }

    /// Returns the amount of the `Balance`.
    pub fn amount(&self) -> u64 {
        self.amount
    }

    /// Safely applies a `BalanceDiff` to the `Balance`.
    pub fn apply_diff(self, diff: &BalanceDiff) -> Result<Self, Error> {
        let amount = (self.amount as i64)
            .checked_add(diff.amount())
            .ok_or_else(|| Error::BalanceOverflow(self.amount as i128 + diff.amount() as i128))?;

        // Given the nature of Utxo, this is not supposed to happen.
        if amount < 0 {
            return Err(Error::NegativeBalance(amount));
        }

        Ok(Self { amount: amount as u64 })
    }
}

impl Packable for Balance {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.amount.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.amount.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let amount = u64::unpack_inner::<R, CHECK>(reader)?;

        Balance::new(amount)
    }
}
