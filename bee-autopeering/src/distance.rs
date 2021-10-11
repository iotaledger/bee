// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{hash, identity::PeerId, salt::Salt};

use prost::bytes::{Buf, Bytes};

pub(crate) type Distance = u32;

pub(crate) const MAX_DISTANCE: Distance = 4294967295;

// From `hive.go`:
//
// returns the distance (uint32) between x and y by xoring the hash of x and y + salt:
// xor(hash(x), hash(y+salt))[:4]

fn distance_with_salt(id1: &PeerId, id2: &PeerId, salt2: Salt) -> Distance {
    let b1 = &id1.public_key().to_bytes();
    let h1 = hash::sha256(b1);

    let b2 = &id2.public_key().to_bytes();
    let s2 = salt2.bytes();
    let h2 = hash::sha256(&join(b2, s2));

    let h_xored = xor(h1, h2);
    let distance = Bytes::copy_from_slice(&h_xored[..4]).get_u32_le();

    distance
}

fn join<const N: usize, const M: usize>(a: &[u8; N], b: &[u8; M]) -> Vec<u8> {
    let mut joined = vec![0u8; a.len() + b.len()];
    joined[0..N].copy_from_slice(a);
    joined[N..M].copy_from_slice(b);
    joined
}

fn xor<const N: usize>(a: [u8; N], b: [u8; N]) -> [u8; N] {
    let mut xored = [0u8; N];
    a.iter()
        .zip(b.iter())
        .enumerate()
        .for_each(|(i, (a, b))| xored[i] = a ^ b);

    xored
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::PeerId;

    #[test]
    fn zero_distance() {
        // let peer1 = PeerId::
    }
}
