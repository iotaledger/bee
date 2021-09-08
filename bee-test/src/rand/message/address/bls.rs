// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::bytes::rand_bytes_array;

use bee_message::address::BlsAddress;

/// Generates a random [`BlsAddress`].
pub fn rand_bls_address() -> BlsAddress {
    BlsAddress::new(rand_bytes_array())
}
