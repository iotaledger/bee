// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rand::Rng;

/// Generates a vector of random bytes with a given length.
pub fn rand_bytes(len: usize) -> Vec<u8> {
    (0..len).map(|_| rand::random::<u8>()).collect()
}

/// Generates an array of random bytes of length 32.
pub fn rand_bytes_array<const N: usize>() -> [u8; N] {
    rand::thread_rng().gen::<[u8; N]>()
}
