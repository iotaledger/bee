// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{milestone::rand_milestone_id, number::rand_number, output::rand_output_id};

use bee_message::input::{Input, TreasuryInput, UtxoInput};

/// Generates a random Utxo input.
pub fn rand_utxo_input() -> UtxoInput {
    rand_output_id().into()
}

/// Generates a random treasury input.
pub fn rand_treasury_input() -> TreasuryInput {
    TreasuryInput::new(rand_milestone_id())
}

/// Generates a random input.
pub fn rand_input() -> Input {
    match rand_number::<u64>() % 2 {
        0 => rand_utxo_input().into(),
        1 => rand_treasury_input().into(),
        _ => unreachable!(),
    }
}
