// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::payload::transaction::Address;

use std::collections::{
    hash_map::{IntoIter, Iter},
    HashMap,
};

#[derive(Debug, Default)]
pub struct BalanceDiff {
    amount: i64,
    dust_allowance: i64,
    dust_output: i64,
}

impl BalanceDiff {
    pub fn new(amount: i64, dust_allowance: i64, dust_output: i64) -> Self {
        Self {
            amount,
            dust_allowance,
            dust_output,
        }
    }

    pub fn amount(&self) -> i64 {
        self.amount
    }

    pub fn dust_allowance(&self) -> i64 {
        self.dust_allowance
    }

    pub fn dust_output(&self) -> i64 {
        self.dust_output
    }
}

#[derive(Debug, Default)]
pub struct BalanceDiffs(HashMap<Address, BalanceDiff>);

impl BalanceDiffs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, other: Self) {
        for (address, entry) in other.0 {
            let e = self.0.entry(address).or_default();
            e.amount = e.amount.saturating_add(entry.amount);
            e.dust_allowance = e.dust_allowance.saturating_add(entry.dust_allowance);
            e.dust_output = e.dust_output.saturating_add(entry.dust_output);
        }
    }

    pub fn get(&self, address: &Address) -> Option<&BalanceDiff> {
        self.0.get(address)
    }

    pub fn amount_add(&mut self, address: Address, amount: u64) {
        let entry = self.0.entry(address).or_default();
        entry.amount = entry.amount.saturating_add(amount as i64);
    }

    pub fn amount_sub(&mut self, address: Address, amount: u64) {
        let entry = self.0.entry(address).or_default();
        entry.amount = entry.amount.saturating_sub(amount as i64);
    }

    pub fn dust_allowance_add(&mut self, address: Address, amount: u64) {
        let entry = self.0.entry(address).or_default();
        entry.dust_allowance = entry.dust_allowance.saturating_add(amount as i64);
    }

    pub fn dust_allowance_sub(&mut self, address: Address, amount: u64) {
        let entry = self.0.entry(address).or_default();
        entry.dust_allowance = entry.dust_allowance.saturating_sub(amount as i64);
    }

    pub fn dust_output_inc(&mut self, address: Address) {
        let entry = self.0.entry(address).or_default();
        entry.dust_output = entry.dust_output.saturating_add(1);
    }

    pub fn dust_output_dec(&mut self, address: Address) {
        let entry = self.0.entry(address).or_default();
        entry.dust_output = entry.dust_output.saturating_sub(1);
    }

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
