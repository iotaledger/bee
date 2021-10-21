// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{error::Error, BalanceDiff};

use bee_common::packable::{Packable, Read, Write};
use bee_message::{constants::IOTA_SUPPLY, output::dust_outputs_max};

/// Holds the balance of an address.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Balance {
    amount: u64,
    dust_allowance: u64,
    dust_outputs: u64,
}

impl Balance {
    /// Creates a new `Balance`.
    pub fn new(amount: u64, dust_allowance: u64, dust_outputs: u64) -> Result<Self, Error> {
        if amount > IOTA_SUPPLY {
            Err(Error::InvalidBalance(amount))
        } else if dust_allowance > IOTA_SUPPLY {
            Err(Error::InvalidBalance(dust_allowance))
        } else if dust_outputs > IOTA_SUPPLY {
            Err(Error::InvalidBalance(dust_outputs))
        } else {
            Ok(Self {
                amount,
                dust_allowance,
                dust_outputs,
            })
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

    /// Returns the number of dust outputs of the `Balance`.
    pub fn dust_outputs(&self) -> u64 {
        self.dust_outputs
    }

    /// Returns whether more dust is allowed on the `Balance`.
    pub fn dust_allowed(&self) -> bool {
        self.dust_outputs() < dust_outputs_max(self.dust_allowance())
    }

    /// Safely applies a `BalanceDiff` to the `Balance`.
    pub fn apply_diff(self, diff: &BalanceDiff) -> Result<Self, Error> {
        let amount = (self.amount as i64)
            .checked_add(diff.amount())
            .ok_or_else(|| Error::BalanceOverflow(self.amount as i128 + diff.amount() as i128))?;
        let dust_allowance = (self.dust_allowance() as i64)
            .checked_add(diff.dust_allowance())
            .ok_or_else(|| Error::BalanceOverflow(self.dust_allowance() as i128 + diff.dust_allowance() as i128))?;
        let dust_outputs = (self.dust_outputs as i64)
            .checked_add(diff.dust_outputs())
            .ok_or_else(|| Error::BalanceOverflow(self.dust_outputs as i128 + diff.dust_outputs() as i128))?;

        // Given the nature of Utxo, this is not supposed to happen.
        if amount < 0 {
            return Err(Error::NegativeBalance(amount));
        }
        if dust_allowance < 0 {
            return Err(Error::NegativeDustAllowance(dust_allowance));
        }
        if dust_outputs < 0 {
            return Err(Error::NegativeDustOutputs(dust_outputs));
        }

        Ok(Self {
            amount: amount as u64,
            dust_allowance: dust_allowance as u64,
            dust_outputs: dust_outputs as u64,
        })
    }
}

impl Packable for Balance {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.amount.packed_len() + self.dust_allowance.packed_len() + self.dust_outputs.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.amount.pack(writer)?;
        self.dust_allowance.pack(writer)?;
        self.dust_outputs.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let amount = u64::unpack_inner::<R, CHECK>(reader)?;
        let dust_allowance = u64::unpack_inner::<R, CHECK>(reader)?;
        let dust_outputs = u64::unpack_inner::<R, CHECK>(reader)?;

        Balance::new(amount, dust_allowance, dust_outputs)
    }
}
