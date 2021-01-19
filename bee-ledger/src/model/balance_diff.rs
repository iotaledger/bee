// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::payload::transaction::Address;

use std::collections::HashMap;

#[derive(Debug)]
pub struct BalanceDiffEntry {
    balance: i64,
    dust_allowance: i64,
    dust_output_count: i64,
}

impl BalanceDiffEntry {
    pub fn new(balance: i64, dust_allowance: i64, dust_output_count: i64) -> Self {
        Self {
            balance,
            dust_allowance,
            dust_output_count,
        }
    }
}

pub struct BalanceDiff(HashMap<Address, BalanceDiffEntry>);
