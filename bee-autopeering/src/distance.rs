// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{hash, identity::PeerId, local::Local, salt::Salt, Peer};

use prost::bytes::{Buf, Bytes};

use std::fmt;

pub(crate) type Distance = u32;

pub(crate) const MAX_DISTANCE: Distance = 4294967295;

pub(crate) struct PeerDistance {
    peer: Peer,
    distance: Distance,
}

impl PeerDistance {
    pub fn from_private_salt(local: Local, peer: Peer) -> Self {
        let distance = salted_distance(
            &local.peer_id(),
            &peer.peer_id(),
            local.private_salt().expect("missing private salt"),
        );

        Self { peer, distance }
    }

    pub fn from_public_salt(local: Local, peer: Peer) -> Self {
        let distance = salted_distance(
            &local.peer_id(),
            &peer.peer_id(),
            local.public_salt().expect("missing public salt"),
        );

        Self { peer, distance }
    }
}

// hive.go:
// returns the distance (uint32) between x and y by xoring the hash of x and (y + salt):
// xor( hash(x), hash(y+salt) )[:4] as little-endian uint32
fn salted_distance(peer1: &PeerId, peer2: &PeerId, salt: Salt) -> Distance {
    let hash1 = hash::sha256(peer1.id_bytes());
    let hash2 = hash::sha256(&concat(peer2.id_bytes(), salt.bytes()));

    let xored = xor(hash1, hash2);

    Bytes::copy_from_slice(&xored[..4]).get_u32_le()
}

fn concat<const N: usize, const M: usize>(bytes1: &[u8; N], bytes2: &[u8; M]) -> Vec<u8> {
    let mut bytes = vec![0u8; bytes1.len() + bytes2.len()]; // N + M
    bytes[0..N].copy_from_slice(bytes1);
    bytes[N..M].copy_from_slice(bytes2);
    bytes
}

fn xor<const N: usize>(a: [u8; N], b: [u8; N]) -> [u8; N] {
    let mut xored = [0u8; N];
    a.iter()
        .zip(b.iter())
        .enumerate()
        .for_each(|(i, (a, b))| xored[i] = a ^ b);

    xored
}

#[derive(Debug, Default)]
pub(crate) struct Neighborhood {
    neighbors: Vec<Distance>,
    size: usize,
}

impl Neighborhood {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            ..Default::default()
        }
    }

    pub fn get_furthest(&self) -> () {}
}

impl fmt::Display for Neighborhood {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.neighbors.len(), self.size)
    }
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
