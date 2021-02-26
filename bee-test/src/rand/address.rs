// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::bytes::rand_bytes;

use bee_message::address::{Address, Ed25519Address};

use std::convert::TryInto;

pub fn rand_ed25519_address() -> Ed25519Address {
    Ed25519Address::new(rand_bytes(32).try_into().unwrap())
}

pub fn rand_address() -> Address {
    // TODO other types
    Address::from(rand_ed25519_address())
}
