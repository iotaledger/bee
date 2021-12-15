// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::Error;

use bee_message::{address::Address, constant::IOTA_SUPPLY, output::Output};

use std::collections::{
    hash_map::{IntoIter, Iter, IterMut},
    HashMap,
};

/// Records a balance difference to apply to an address.
#[derive(Clone, Debug, Default)]
pub struct BalanceDiff(i64);

impl BalanceDiff {
    /// Creates a new [`BalanceDiff`].
    pub fn new(amount: i64) -> Result<Self, Error> {
        if amount.abs() as u64 > IOTA_SUPPLY {
            Err(Error::InvalidBalanceDiff(amount))
        } else {
            Ok(Self(amount))
        }
    }

    /// Returns the amount of a [`BalanceDiff`].
    pub fn amount(&self) -> i64 {
        self.0
    }

    /// Negates a [`BalanceDiff`].
    pub fn negate(&mut self) {
        // TODO this can overflow
        self.0 = -self.0;
    }
}

/// Records a balance differences to apply to addresses.
#[derive(Clone, Debug, Default)]
pub struct BalanceDiffs(HashMap<Address, BalanceDiff>);

impl BalanceDiffs {
    /// Creates a new [`BalanceDiffs`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Merges a [`BalanceDiffs`] into another.
    pub fn merge(&mut self, other: Self) -> Result<(), Error> {
        for (address, diff) in other.0 {
            let e = self.0.entry(address).or_default();
            e.0 =
                e.0.checked_add(diff.0)
                    .ok_or(Error::BalanceDiffOverflow(e.0 as i128 + diff.0 as i128))?;
        }

        Ok(())
    }

    /// Gets the [`BalanceDiff`] of a given address.
    pub fn get(&self, address: &Address) -> Option<&BalanceDiff> {
        self.0.get(address)
    }

    /// Negates a [`BalanceDiffs`].
    pub fn negate(&mut self) {
        for (_, diff) in self.iter_mut() {
            diff.negate();
        }
    }

    /// Creates a new negated version of a [`BalanceDiffs`].
    pub fn negated(&self) -> Self {
        let mut new = self.clone();
        new.negate();
        new
    }

    /// Adds an output to a [`BalanceDiffs`].
    pub fn output_add(&mut self, output: &Output) -> Result<(), Error> {
        match output {
            Output::Simple(output) => {
                self.amount_add(*output.address(), output.amount())?;
            }
            Output::Treasury(_) => return Err(Error::UnsupportedOutputKind(output.kind())),
            Output::Extended(output) => {
                self.amount_add(*output.address(), output.amount())?;
            }
            Output::Alias(_output) => {
                // TODO
            }
            Output::Foundry(output) => {
                self.amount_add(*output.address(), output.amount())?;
            }
            Output::Nft(output) => {
                self.amount_add(*output.address(), output.amount())?;
            }
        }

        Ok(())
    }

    /// Subtracts an output from a BalanceDiffs`.
    pub fn output_sub(&mut self, output: &Output) -> Result<(), Error> {
        match output {
            Output::Simple(output) => {
                self.amount_sub(*output.address(), output.amount())?;
            }
            Output::Treasury(_) => return Err(Error::UnsupportedOutputKind(output.kind())),
            Output::Extended(output) => {
                self.amount_sub(*output.address(), output.amount())?;
            }
            Output::Alias(_output) => {
                // TODO
            }
            Output::Foundry(output) => {
                self.amount_sub(*output.address(), output.amount())?;
            }
            Output::Nft(output) => {
                self.amount_sub(*output.address(), output.amount())?;
            }
        }

        Ok(())
    }

    /// Adds a given amount to a given address.
    pub fn amount_add(&mut self, address: Address, amount: u64) -> Result<(), Error> {
        let entry = self.0.entry(address).or_default();
        entry.0 = entry
            .0
            .checked_add(amount as i64)
            .ok_or(Error::BalanceDiffOverflow(entry.0 as i128 + amount as i128))?;
        Ok(())
    }

    /// Subtracts a given amount from a given address.
    pub fn amount_sub(&mut self, address: Address, amount: u64) -> Result<(), Error> {
        let entry = self.0.entry(address).or_default();
        entry.0 = entry
            .0
            .checked_sub(amount as i64)
            .ok_or(Error::BalanceDiffOverflow(entry.0 as i128 + amount as i128))?;
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
