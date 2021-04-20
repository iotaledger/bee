// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::integer::rand_integer_range;

use bee_ledger::types::Balance;
use bee_message::constants::IOTA_SUPPLY;

pub fn rand_balance() -> Balance {
    Balance::new(
        rand_integer_range(0..IOTA_SUPPLY),
        rand_integer_range(0..IOTA_SUPPLY),
        rand_integer_range(0..IOTA_SUPPLY),
    )
    .unwrap()
}
