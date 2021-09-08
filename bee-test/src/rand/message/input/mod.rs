// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod utxo;

use crate::rand::number::rand_number;

pub use utxo::rand_utxo_input;

use bee_message::input::{Input, UtxoInput};

/// Generates a random [`Input`].
#[allow(clippy::modulo_one)]
pub fn rand_input() -> Input {
    match rand_number::<u8>() % 1 {
        UtxoInput::KIND => rand_utxo_input().into(),
        _ => unreachable!(),
    }
}
