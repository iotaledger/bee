// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::integer::rand_integer;

use bee_ledger::balance::Balance;

pub fn rand_balance() -> Balance {
    Balance::new(rand_integer(), rand_integer(), rand_integer())
}
