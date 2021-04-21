// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::output::rand_output_id;

use bee_message::input::{Input, UtxoInput};

pub fn rand_input() -> Input {
    // TODO add other kind of inputs
    rand_utxo_input().into()
}

pub fn rand_utxo_input() -> UtxoInput {
    rand_output_id().into()
}
