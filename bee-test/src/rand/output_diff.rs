// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    integer::rand_integer_range, milestone::rand_milestone_id, option::rand_option, output::rand_output_id,
};

use bee_ledger::types::{OutputDiff, TreasuryDiff};

pub fn rand_treasury_diff() -> TreasuryDiff {
    TreasuryDiff::new(rand_milestone_id(), rand_milestone_id())
}

pub fn rand_output_diff() -> OutputDiff {
    let spent_outputs_len = rand_integer_range::<usize, _>(0..10);
    let mut spent_outputs = Vec::new();
    let created_outputs_len = rand_integer_range::<usize, _>(0..10);
    let mut created_outputs = Vec::new();

    for _ in 0..spent_outputs_len {
        spent_outputs.push(rand_output_id());
    }

    for _ in 0..created_outputs_len {
        created_outputs.push(rand_output_id());
    }

    OutputDiff::new(spent_outputs, created_outputs, rand_option(rand_treasury_diff()))
}
