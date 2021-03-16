// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_crypto::ternary::{
    sponge::{CurlP81, Sponge},
    HASH_LENGTH,
};
use bee_ternary::{b1t6, Btrit, T1B1Buf, TritBuf, Trits, T1B1};

use crypto::hashes::{blake2b::Blake2b256, Digest};

pub fn compute_pow_score(bytes: &[u8]) -> f64 {
    pow_score(&pow_hash(bytes), bytes.len())
}

pub fn pow_hash(bytes: &[u8]) -> TritBuf<T1B1Buf> {
    assert!(bytes.len() >= std::mem::size_of::<u8>());

    let mut curl = CurlP81::new();
    let length = bytes.len() - std::mem::size_of::<u64>();
    let mut pow_input = TritBuf::<T1B1Buf>::with_capacity(HASH_LENGTH);
    let pow_digest = Blake2b256::digest(&bytes[..length]);

    b1t6::encode::<T1B1Buf>(&pow_digest)
        .iter()
        .for_each(|t| pow_input.push(t));
    b1t6::encode::<T1B1Buf>(&bytes[length..])
        .iter()
        .for_each(|t| pow_input.push(t));
    pow_input.push(Btrit::Zero);
    pow_input.push(Btrit::Zero);
    pow_input.push(Btrit::Zero);

    curl.digest(&pow_input).unwrap()
}

pub fn pow_score(pow_hash: &Trits<T1B1>, len: usize) -> f64 {
    let zeros = pow_hash.iter().rev().take_while(|t| *t == Btrit::Zero).count() as u32;

    3u128.pow(zeros) as f64 / len as f64
}
