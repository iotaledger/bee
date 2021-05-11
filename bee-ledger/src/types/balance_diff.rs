// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::Error;

use bee_message::{
    address::Address,
    constants::IOTA_SUPPLY,
    output::{Output, DUST_THRESHOLD},
};

use std::collections::{
    hash_map::{IntoIter, Iter, IterMut},
    HashMap,
};

/// Records a balance difference to apply to an address.
#[derive(Clone, Debug, Default)]
pub struct BalanceDiff {
    amount: i64,
    dust_allowance: i64,
    dust_outputs: i64,
}

impl BalanceDiff {
    /// Creates a new `BalanceDiff`.
    pub fn new(amount: i64, dust_allowance: i64, dust_outputs: i64) -> Result<Self, Error> {
        if amount.abs() as u64 > IOTA_SUPPLY {
            Err(Error::InvalidBalanceDiff(amount))
        } else if dust_allowance.abs() as u64 > IOTA_SUPPLY {
            Err(Error::InvalidBalanceDiff(dust_allowance))
        } else if dust_outputs.abs() as u64 > IOTA_SUPPLY {
            Err(Error::InvalidBalanceDiff(dust_outputs))
        } else {
            Ok(Self {
                amount,
                dust_allowance,
                dust_outputs,
            })
        }
    }

    /// Returns the amount of a `BalanceDiff`.
    pub fn amount(&self) -> i64 {
        self.amount
    }

    /// Returns the dust allowance of a `BalanceDiff`.
    pub fn dust_allowance(&self) -> i64 {
        self.dust_allowance
    }

    /// Returns the number of dust outputs of a `BalanceDiff`.
    pub fn dust_outputs(&self) -> i64 {
        self.dust_outputs
    }

    /// Returns whether dust allowance has been decreased or dust outputs has been increased.
    pub fn is_dust_mutating(&self) -> bool {
        self.dust_allowance < 0 || self.dust_outputs > 0
    }

    /// Negates a `BalanceDiff`.
    pub fn negate(&mut self) {
        self.amount = -self.amount;
        self.dust_allowance = -self.dust_allowance;
        self.dust_outputs = -self.dust_outputs;
    }
}

/// Records a balance differences to apply to addresses.
#[derive(Clone, Debug, Default)]
pub struct BalanceDiffs(HashMap<Address, BalanceDiff>);

impl BalanceDiffs {
    /// Creates a new `BalanceDiffs`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Merges a `BalanceDiffs` into another.
    pub fn merge(&mut self, other: Self) -> Result<(), Error> {
        for (address, diff) in other.0 {
            let e = self.0.entry(address).or_default();
            e.amount = e
                .amount
                .checked_add(diff.amount)
                .ok_or_else(|| Error::BalanceDiffOverflow(e.amount as i128 + diff.amount as i128))?;
            e.dust_allowance = e
                .dust_allowance
                .checked_add(diff.dust_allowance)
                .ok_or_else(|| Error::BalanceDiffOverflow(e.dust_allowance as i128 + diff.dust_allowance as i128))?;
            e.dust_outputs = e
                .dust_outputs
                .checked_add(diff.dust_outputs)
                .ok_or_else(|| Error::BalanceDiffOverflow(e.dust_outputs as i128 + diff.dust_outputs as i128))?;
        }

        Ok(())
    }

    /// Gets the `BalanceDiff` of a given address.
    pub fn get(&self, address: &Address) -> Option<&BalanceDiff> {
        self.0.get(address)
    }

    /// Negates a `BalanceDiffs`.
    pub fn negate(&mut self) {
        for (_, diff) in self.iter_mut() {
            diff.negate();
        }
    }

    /// Creates a new negated version of a `BalanceDiffs`.
    pub fn negated(&self) -> Self {
        let mut new = self.clone();
        new.negate();
        new
    }

    /// Adds an output to a `BalanceDiffs`.
    pub fn output_add(&mut self, output: &Output) -> Result<(), Error> {
        match output {
            Output::SignatureLockedSingle(output) => {
                self.amount_add(*output.address(), output.amount())?;
                if output.amount() < DUST_THRESHOLD {
                    self.dust_outputs_inc(*output.address())?;
                }
            }
            Output::SignatureLockedDustAllowance(output) => {
                self.amount_add(*output.address(), output.amount())?;
                self.dust_allowance_add(*output.address(), output.amount())?;
            }
            output => return Err(Error::UnsupportedOutputKind(output.kind())),
        }

        Ok(())
    }

    /// Subtracts an output from a BalanceDiffs`.
    pub fn output_sub(&mut self, output: &Output) -> Result<(), Error> {
        match output {
            Output::SignatureLockedSingle(output) => {
                self.amount_sub(*output.address(), output.amount())?;
                if output.amount() < DUST_THRESHOLD {
                    self.dust_outputs_dec(*output.address())?;
                }
            }
            Output::SignatureLockedDustAllowance(output) => {
                self.amount_sub(*output.address(), output.amount())?;
                self.dust_allowance_sub(*output.address(), output.amount())?;
            }
            output => return Err(Error::UnsupportedOutputKind(output.kind())),
        }

        Ok(())
    }

    /// Adds a given amount to a given address.
    pub fn amount_add(&mut self, address: Address, amount: u64) -> Result<(), Error> {
        let entry = self.0.entry(address).or_default();
        entry.amount = entry
            .amount
            .checked_add(amount as i64)
            .ok_or_else(|| Error::BalanceDiffOverflow(entry.amount as i128 + amount as i128))?;
        Ok(())
    }

    /// Subtracts a given amount from a given address.
    pub fn amount_sub(&mut self, address: Address, amount: u64) -> Result<(), Error> {
        let entry = self.0.entry(address).or_default();
        entry.amount = entry
            .amount
            .checked_sub(amount as i64)
            .ok_or_else(|| Error::BalanceDiffOverflow(entry.amount as i128 + amount as i128))?;
        Ok(())
    }

    /// Adds a given dust allowance to a given address.
    pub fn dust_allowance_add(&mut self, address: Address, amount: u64) -> Result<(), Error> {
        let entry = self.0.entry(address).or_default();
        entry.dust_allowance = entry
            .dust_allowance
            .checked_add(amount as i64)
            .ok_or_else(|| Error::BalanceDiffOverflow(entry.dust_allowance as i128 + amount as i128))?;
        Ok(())
    }

    /// Subtracts a given dust allowance from a given address.
    pub fn dust_allowance_sub(&mut self, address: Address, amount: u64) -> Result<(), Error> {
        let entry = self.0.entry(address).or_default();
        entry.dust_allowance = entry
            .dust_allowance
            .checked_sub(amount as i64)
            .ok_or_else(|| Error::BalanceDiffOverflow(entry.dust_allowance as i128 + amount as i128))?;
        Ok(())
    }

    /// Increments the number of dust outputs of a given address.
    pub fn dust_outputs_inc(&mut self, address: Address) -> Result<(), Error> {
        let entry = self.0.entry(address).or_default();
        entry.dust_outputs = entry
            .dust_outputs
            .checked_add(1)
            .ok_or_else(|| Error::BalanceDiffOverflow(entry.dust_outputs as i128 + 1))?;
        Ok(())
    }

    /// Decrements the number of dust outputs of a given address.
    pub fn dust_outputs_dec(&mut self, address: Address) -> Result<(), Error> {
        let entry = self.0.entry(address).or_default();
        entry.dust_outputs = entry
            .dust_outputs
            .checked_sub(1)
            .ok_or_else(|| Error::BalanceDiffOverflow(entry.dust_outputs as i128 + 1))?;
        Ok(())
    }

    /// Creates an iterator over the balance diffs.
    pub fn iter(&self) -> Iter<'_, Address, BalanceDiff> {
        self.0.iter()
    }

    /// Creates a mutable iterator over the balance diffs.
    pub fn iter_mut(&mut self) -> IterMut<'_, Address, BalanceDiff> {
        self.0.iter_mut()
    }
}

impl IntoIterator for BalanceDiffs {
    type Item = (Address, BalanceDiff);
    type IntoIter = IntoIter<Address, BalanceDiff>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
