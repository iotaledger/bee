// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{integer::rand_integer_range, milestone::rand_milestone_index, transaction::rand_transaction_id};

use bee_ledger::types::{ConsumedOutput, Unspent};
use bee_message::{constants::INPUT_OUTPUT_INDEX_RANGE, output::OutputId};

pub fn rand_output_id() -> OutputId {
    OutputId::new(rand_transaction_id(), rand_integer_range(INPUT_OUTPUT_INDEX_RANGE)).unwrap()
}

pub fn rand_unspent_output_id() -> Unspent {
    Unspent::new(rand_output_id())
}

pub fn rand_consumed_output() -> ConsumedOutput {
    ConsumedOutput::new(rand_transaction_id(), rand_milestone_index())
}
