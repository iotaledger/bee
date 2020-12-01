// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::output::random_output_id;

use bee_message::payload::transaction::{Input, UTXOInput};

pub fn random_input() -> Input {
    // TODO add other kind of inputs
    random_utxo_input().into()
}

pub fn random_utxo_input() -> UTXOInput {
    random_output_id().into()
}
