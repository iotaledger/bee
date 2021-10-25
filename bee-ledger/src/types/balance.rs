// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{error::Error, BalanceDiff};

use bee_message::{constants::IOTA_SUPPLY, output::dust_outputs_max};
use bee_packable::{
    bounded::{BoundedU64, InvalidBoundedU64},
    Packable,
};

use std::convert::TryInto;

fn invalid_bounded_int_to_invalid_balance(err: InvalidBoundedU64<0, IOTA_SUPPLY>) -> Error {
    Error::InvalidBalance(err.0)
}

/// Holds the balance of an address.
#[derive(Clone, Debug, Default, Eq, PartialEq, Packable)]
#[packable(unpack_error = Error, with = invalid_bounded_int_to_invalid_balance)]
pub struct Balance {
    amount: BoundedU64<0, IOTA_SUPPLY>,
    dust_allowance: BoundedU64<0, IOTA_SUPPLY>,
    dust_outputs: BoundedU64<0, IOTA_SUPPLY>,
}

impl Balance {
    /// Creates a new `Balance`.
    pub fn new(amount: u64, dust_allowance: u64, dust_outputs: u64) -> Result<Self, Error> {
        Ok(Self {
            amount: amount.try_into().map_err(invalid_bounded_int_to_invalid_balance)?,
            dust_allowance: dust_allowance
                .try_into()
                .map_err(invalid_bounded_int_to_invalid_balance)?,
            dust_outputs: dust_outputs
                .try_into()
                .map_err(invalid_bounded_int_to_invalid_balance)?,
        })
    }

    /// Returns the amount of the `Balance`.
    pub fn amount(&self) -> u64 {
        self.amount.into()
    }

    /// Returns the dust allowance of the `Balance`.
    pub fn dust_allowance(&self) -> u64 {
        self.dust_allowance.into()
    }

    /// Returns the number of dust outputs of the `Balance`.
    pub fn dust_outputs(&self) -> u64 {
        self.dust_outputs.into()
    }

    /// Returns whether more dust is allowed on the `Balance`.
    pub fn dust_allowed(&self) -> bool {
        self.dust_outputs() < dust_outputs_max(self.dust_allowance())
    }

    /// Safely applies a `BalanceDiff` to the `Balance`.
    pub fn apply_diff(self, diff: &BalanceDiff) -> Result<Self, Error> {
        let amount = (self.amount() as i64)
            .checked_add(diff.amount())
            .ok_or_else(|| Error::BalanceOverflow(self.amount() as i128 + diff.amount() as i128))?;
        let dust_allowance = (self.dust_allowance() as i64)
            .checked_add(diff.dust_allowance())
            .ok_or_else(|| Error::BalanceOverflow(self.dust_allowance() as i128 + diff.dust_allowance() as i128))?;
        let dust_outputs = (self.dust_outputs() as i64)
            .checked_add(diff.dust_outputs())
            .ok_or_else(|| Error::BalanceOverflow(self.dust_outputs() as i128 + diff.dust_outputs() as i128))?;

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

        Self::new(amount as u64, dust_allowance as u64, dust_outputs as u64)
    }
}
