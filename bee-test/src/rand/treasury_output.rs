// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{integer::rand_integer_range, milestone::rand_milestone_id};

use bee_ledger::types::TreasuryOutput;
use bee_message::output::{self, TREASURY_OUTPUT_AMOUNT};

pub fn rand_treasury_output() -> TreasuryOutput {
    TreasuryOutput::new(
        // TODO move
        output::TreasuryOutput::new(rand_integer_range(TREASURY_OUTPUT_AMOUNT)).unwrap(),
        rand_milestone_id(),
    )
}
