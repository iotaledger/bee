// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod bls;
mod ed25519;

use crate::rand::number::rand_number;

pub use bls::rand_bls_address;
pub use ed25519::rand_ed25519_address;

use bee_message::address::{Address, BlsAddress, Ed25519Address};

/// Generates a random [`Address`].
pub fn rand_address() -> Address {
    match rand_number::<u8>() % 2 {
        Ed25519Address::KIND => rand_ed25519_address().into(),
        BlsAddress::KIND => rand_bls_address().into(),
        _ => unreachable!(),
    }
}
