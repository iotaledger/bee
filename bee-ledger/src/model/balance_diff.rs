// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::payload::transaction::Address;

use std::collections::{hash_map::IntoIter, HashMap};

#[derive(Debug, Default)]
pub struct BalanceDiffEntry {
    pub(crate) balance: i64,
    pub(crate) dust_allowance: i64,
    pub(crate) dust_output: i64,
}

impl BalanceDiffEntry {
    pub fn new(balance: i64, dust_allowance: i64, dust_output: i64) -> Self {
        Self {
            balance,
            dust_allowance,
            dust_output,
        }
    }
}

#[derive(Debug, Default)]
pub struct BalanceDiff(HashMap<Address, BalanceDiffEntry>);

impl BalanceDiff {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, address: &Address) -> Option<&BalanceDiffEntry> {
        self.0.get(address)
    }

    pub fn balance_add(&mut self, address: Address, amount: u64) {
        let entry = self.0.entry(address).or_default();
        entry.balance = entry.balance.saturating_add(amount as i64);
    }

    pub fn balance_sub(&mut self, address: Address, amount: u64) {
        let entry = self.0.entry(address).or_default();
        entry.balance = entry.balance.saturating_sub(amount as i64);
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
}

impl IntoIterator for BalanceDiff {
    type Item = (Address, BalanceDiffEntry);
    type IntoIter = IntoIter<Address, BalanceDiffEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
