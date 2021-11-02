// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{bytes::rand_bytes_32, number::rand_number};

use bee_message::address::{Address, Ed25519Address};

/// Generates a random ED25519 address.
pub fn rand_ed25519_address() -> Ed25519Address {
    Ed25519Address::new(rand_bytes_32())
}

/// Generates a random address.
pub fn rand_address() -> Address {
    #[allow(clippy::modulo_one)]
    Address::from(match rand_number::<u64>() % 1 {
        0 => rand_ed25519_address(),
        _ => unreachable!(),
    })
}
