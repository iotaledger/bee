// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::Balance;
use bee_message::constants::IOTA_SUPPLY;

use crate::rand::number::rand_number_range;

/// Generates a random balance.
pub fn rand_balance() -> Balance {
    Balance::new(
        rand_number_range(0..=IOTA_SUPPLY),
        rand_number_range(0..=IOTA_SUPPLY),
        rand_number_range(0..=IOTA_SUPPLY),
    )
    .unwrap()
}
