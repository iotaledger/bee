// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{integer::rand_integer_range, transaction::rand_transaction_id};

use bee_message::{constants::INPUT_OUTPUT_INDEX_RANGE, output::OutputId};

pub fn rand_output_id() -> OutputId {
    OutputId::new(rand_transaction_id(), rand_integer_range(INPUT_OUTPUT_INDEX_RANGE)).unwrap()
}
