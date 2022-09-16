// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crypto::hashes::{blake2b::Blake2b256, Digest};

/// Hashes a string network name to a digit network ID.
pub fn network_name_to_id(network_name: &str) -> u64 {
    // PANIC: indexing and unwrapping is fine as a Blake2b256 digest has 32 bytes, we ask for 8 of them and we convert
    // that slice to an array of 8 bytes.
    u64::from_le_bytes(Blake2b256::digest(network_name.as_bytes())[0..8].try_into().unwrap())
}
