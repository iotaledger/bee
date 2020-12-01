// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{integer::random_integer_range, transaction::random_transaction_id};

use bee_message::payload::transaction::{OutputId, INPUT_OUTPUT_INDEX_RANGE};

pub fn random_output_id() -> OutputId {
    OutputId::new(random_transaction_id(), random_integer_range(INPUT_OUTPUT_INDEX_RANGE)).unwrap()
}
