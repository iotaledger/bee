// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::address::Address;

use std::collections::{
    hash_map::{IntoIter, Iter},
    HashMap,
};

/// Records a balance difference to apply to an address.
#[derive(Debug, Default)]
pub struct BalanceDiff {
    amount: i64,
    dust_allowance: i64,
    dust_output: i64,
}

impl BalanceDiff {
    /// Creates a new `BalanceDiff`.
    pub fn new(amount: i64, dust_allowance: i64, dust_output: i64) -> Self {
        Self {
            amount,
            dust_allowance,
            dust_output,
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

    /// Returns the dust output of a `BalanceDiff`.
    pub fn dust_output(&self) -> i64 {
        self.dust_output
    }

    /// Dust validation rules need to be check if
    ///    dust allowance has been decreased
    /// || dust output has been increased.
    pub fn is_dust_mutating(&self) -> bool {
        self.dust_allowance < 0 || self.dust_output > 0
    }
}

/// Records a balance differences to apply to addresses.
#[derive(Debug, Default)]
pub struct BalanceDiffs(HashMap<Address, BalanceDiff>);

impl BalanceDiffs {
    /// Creates a new `BalanceDiffs`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Merges a `BalanceDiffs` into another.
    pub fn merge(&mut self, other: Self) {
        for (address, diff) in other.0 {
            let e = self.0.entry(address).or_default();
            e.amount = e.amount.saturating_add(diff.amount);
            e.dust_allowance = e.dust_allowance.saturating_add(diff.dust_allowance);
            e.dust_output = e.dust_output.saturating_add(diff.dust_output);
        }
    }

    /// Gets the `BalanceDiff` of a given address.
    pub fn get(&self, address: &Address) -> Option<&BalanceDiff> {
        self.0.get(address)
    }

    /// Adds a given amount to a given address.
    pub fn amount_add(&mut self, address: Address, amount: u64) {
        let entry = self.0.entry(address).or_default();
        entry.amount = entry.amount.saturating_add(amount as i64);
    }

    /// Subtracts a given amount from a given address.
    pub fn amount_sub(&mut self, address: Address, amount: u64) {
        let entry = self.0.entry(address).or_default();
        entry.amount = entry.amount.saturating_sub(amount as i64);
    }

    /// Adds a given dust allowance to a given address.
    pub fn dust_allowance_add(&mut self, address: Address, amount: u64) {
        let entry = self.0.entry(address).or_default();
        entry.dust_allowance = entry.dust_allowance.saturating_add(amount as i64);
    }

    /// Subtracts a given dust allowance from a given address.
    pub fn dust_allowance_sub(&mut self, address: Address, amount: u64) {
        let entry = self.0.entry(address).or_default();
        entry.dust_allowance = entry.dust_allowance.saturating_sub(amount as i64);
    }

    /// Increments the number of dust outputs of a given address.
    pub fn dust_output_inc(&mut self, address: Address) {
        let entry = self.0.entry(address).or_default();
        entry.dust_output = entry.dust_output.saturating_add(1);
    }

    /// Decrements the number of dust outputs of a given address.
    pub fn dust_output_dec(&mut self, address: Address) {
        let entry = self.0.entry(address).or_default();
        entry.dust_output = entry.dust_output.saturating_sub(1);
    }

    /// Creates an iterator over the balance diffs.
    pub fn iter(&self) -> Iter<'_, Address, BalanceDiff> {
        self.0.iter()
    }
}

impl IntoIterator for BalanceDiffs {
    type Item = (Address, BalanceDiff);
    type IntoIter = IntoIter<Address, BalanceDiff>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
