// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{integer::rand_integer_range, output::rand_output_id};

use bee_ledger::model::OutputDiff;

pub fn rand_output_diff() -> OutputDiff {
    let spent_outputs_len = rand_integer_range::<usize>(0..10);
    let mut spent_outputs = Vec::new();
    let created_outputs_len = rand_integer_range::<usize>(0..10);
    let mut created_outputs = Vec::new();

    for _ in 0..spent_outputs_len {
        spent_outputs.push(rand_output_id());
    }

    for _ in 0..created_outputs_len {
        created_outputs.push(rand_output_id());
    }

    OutputDiff::new(spent_outputs, created_outputs)
}
