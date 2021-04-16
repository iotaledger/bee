// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::milestone::rand_milestone_id;

use bee_message::input::{Input, TreasuryInput};

/// Generates a random treasury input.
pub fn rand_treasury_input() -> Input {
    TreasuryInput::new(rand_milestone_id()).into()
}
