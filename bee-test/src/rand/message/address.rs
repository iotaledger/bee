// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{bytes::rand_bytes_array, number::rand_number};

use bee_message::address::{Address, BlsAddress, Ed25519Address};

/// Generates a random [`Ed25519Address`].
pub fn rand_ed25519_address() -> Ed25519Address {
    Ed25519Address::new(rand_bytes_array())
}

/// Generates a random [`BlsAddress`].
pub fn rand_bls_address() -> BlsAddress {
    BlsAddress::new(rand_bytes_array())
}

/// Generates a random [`Address`].
pub fn rand_address() -> Address {
    match rand_number::<u8>() % 2 {
        Ed25519Address::KIND => rand_ed25519_address().into(),
        BlsAddress::KIND => rand_bls_address().into(),
        _ => unreachable!(),
    }
}
