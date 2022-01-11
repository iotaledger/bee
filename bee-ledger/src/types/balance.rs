// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{error::Error, BalanceDiff};

use bee_message::constant::IOTA_SUPPLY;

/// Holds the balance of an address.
#[derive(Clone, Debug, Default, Eq, PartialEq, bee_packable::Packable)]
pub struct Balance(u64);

impl Balance {
    /// Creates a new [`Balance`].
    pub fn new(amount: u64) -> Result<Self, Error> {
        if amount > IOTA_SUPPLY {
            Err(Error::InvalidBalance(amount))
        } else {
            Ok(Self(amount))
        }
    }

    /// Returns the amount of the [`Balance`].
    pub fn amount(&self) -> u64 {
        self.0
    }

    /// Safely applies a [`BalanceDiff`] to the [`Balance`].
    pub fn apply_diff(self, diff: &BalanceDiff) -> Result<Self, Error> {
        let amount = (self.0 as i64)
            .checked_add(diff.amount())
            .ok_or_else(|| Error::BalanceOverflow(self.0 as i128 + diff.amount() as i128))?;

        // Given the nature of Utxo, this is not supposed to happen.
        if amount < 0 {
            return Err(Error::NegativeBalance(amount));
        }

        Ok(Self(amount as u64))
    }
}
