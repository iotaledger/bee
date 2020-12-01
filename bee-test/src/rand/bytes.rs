// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rand::Rng;

pub fn random_bytes(len: usize) -> Vec<u8> {
    (0..len).map(|_| rand::random::<u8>()).collect::<Vec<u8>>()
}

pub fn random_bytes_32() -> [u8; 32] {
    rand::thread_rng().gen::<[u8; 32]>()
}
