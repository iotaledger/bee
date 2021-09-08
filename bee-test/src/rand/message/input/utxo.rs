// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::message::output::rand_output_id;

use bee_message::input::UtxoInput;

/// Generates a random [`UtxoInput`].
pub fn rand_utxo_input() -> UtxoInput {
    rand_output_id().into()
}
