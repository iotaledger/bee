// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::integer::rand_integer_range;

use bee_message::output::{Output, TreasuryOutput, TREASURY_OUTPUT_AMOUNT};

/// Generates a random treasury output.
pub fn rand_treasury_output() -> Output {
    TreasuryOutput::new(rand_integer_range(TREASURY_OUTPUT_AMOUNT))
        .unwrap()
        .into()
}
