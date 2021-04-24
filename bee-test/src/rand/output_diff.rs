// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    milestone::rand_milestone_id, number::rand_number_range, option::rand_option, output::rand_output_id,
};

use bee_ledger::types::{OutputDiff, TreasuryDiff};

/// Generates a random treasury diff.
pub fn rand_treasury_diff() -> TreasuryDiff {
    TreasuryDiff::new(rand_milestone_id(), rand_milestone_id())
}

/// Generates a random output diff.
pub fn rand_output_diff() -> OutputDiff {
    let consumed_outputs_len = rand_number_range(0..100);
    let mut spent_outputs = Vec::new();
    let created_outputs_len = rand_number_range(0..100);
    let mut created_outputs = Vec::new();

    for _ in 0..consumed_outputs_len {
        spent_outputs.push(rand_output_id());
    }

    for _ in 0..created_outputs_len {
        created_outputs.push(rand_output_id());
    }

    OutputDiff::new(spent_outputs, created_outputs, rand_option(rand_treasury_diff()))
}
