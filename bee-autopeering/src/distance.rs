// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{hash, identity::PeerId, local::Local, peer::Peer, salt::Salt};

use prost::bytes::{Buf, Bytes};

use std::{
    cmp,
    collections::{BTreeMap, BTreeSet},
    fmt,
    fmt::Display,
    mem,
};

pub(crate) type Distance = u32;

pub(crate) const MAX_DISTANCE: Distance = 4294967295;
pub(crate) const MAX_NEIGHBORHOOD_SIZE_INBOUND: usize = 4;
pub(crate) const MAX_NEIGHBORHOOD_SIZE_OUTBOUND: usize = 4;

#[derive(Debug)]
pub(crate) struct PeerDistance {
    peer: Peer,
    distance: Distance,
}

impl PeerDistance {
    pub fn from_private_salt(local: &Local, peer: Peer) -> Self {
        let distance = salted_distance(
            &local.peer_id(),
            &peer.peer_id(),
            &local.private_salt().expect("missing private salt"),
        );

        Self { peer, distance }
    }

    pub fn from_public_salt(local: &Local, peer: Peer) -> Self {
        let distance = salted_distance(
            &local.peer_id(),
            &peer.peer_id(),
            &local.public_salt().expect("missing public salt"),
        );

        Self { peer, distance }
    }

    pub fn peer(&self) -> &Peer {
        &self.peer
    }

    pub fn distance(&self) -> Distance {
        self.distance
    }
}

impl Display for PeerDistance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.peer().peer_id(), self.distance())
    }
}

impl Eq for PeerDistance {}
impl PartialEq for PeerDistance {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}
impl PartialOrd for PeerDistance {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(&other))
    }
}
impl Ord for PeerDistance {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.distance.cmp(&other.distance)
    }
}

#[derive(Debug, Default)]
pub(crate) struct Neighborhood<const N: usize> {
    local: Local,
    neighbors: Vec<PeerDistance>,
}

impl<const N: usize> Neighborhood<N> {
    pub(crate) fn new(local: Local) -> Self {
        Self {
            local,
            neighbors: Vec::with_capacity(N),
        }
    }

    pub(crate) fn insert_peer(&mut self, peer: Peer) -> bool {
        // If the peer already exists remove it.
        // NOTE: It's a bit less efficient doing it like this, but the code requires less branching this way.
        let _ = self.remove_peer(&peer.peer_id());

        if self.neighbors.len() >= N {
            return false;
        }

        // Calculate the distance to that peer.
        let distance = salted_distance(
            &self.local.peer_id(),
            &peer.peer_id(),
            &self.local.private_salt().expect("missing private salt"),
        );

        self.neighbors.push(PeerDistance { distance, peer });

        true
    }

    pub(crate) fn remove_peer(&mut self, peer_id: &PeerId) -> Option<Peer> {
        if self.neighbors.is_empty() {
            None
        } else if let Some(index) = self.neighbors.iter().position(|pd| &pd.peer().peer_id() == peer_id) {
            let PeerDistance { peer, .. } = self.neighbors.remove(index);
            Some(peer)
        } else {
            None
        }
    }

    // From the candidate list pick the first, that is closer than the currently furthest neighbor.
    pub(crate) fn select_candidate<'a>(&mut self, candidates: &'a [PeerDistance]) -> Option<&'a PeerDistance> {
        if candidates.is_empty() {
            None
        } else if let Some(furthest) = self.get_furthest() {
            for candidate in candidates {
                if candidate < furthest {
                    return Some(candidate);
                }
            }
            None
        } else {
            None
        }
    }

    pub(crate) fn get_furthest(&mut self) -> Option<&PeerDistance> {
        if self.neighbors.len() < N {
            None
        } else {
            self.neighbors.sort_unstable();
            self.neighbors.last()
        }
    }

    pub(crate) fn update_distances(&mut self) {
        let local_peer_id = self.local.peer_id();
        let local_private_salt = self.local.private_salt().expect("missing private salt");

        self.neighbors.iter_mut().for_each(move |pd| {
            pd.distance = salted_distance(&local_peer_id, &pd.peer().peer_id(), &local_private_salt);
        });
    }

    pub(crate) fn is_full(&self) -> bool {
        self.neighbors.len() == N
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.neighbors.is_empty()
    }

    pub(crate) fn num_neighbors(&self) -> usize {
        self.neighbors.len()
    }
}

impl<const N: usize> fmt::Display for Neighborhood<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.neighbors.len(), N)
    }
}

// hive.go:
// returns the distance (uint32) between x and y by xoring the hash of x and (y + salt):
// xor( hash(x), hash(y+salt) )[:4] as little-endian uint32
fn salted_distance(peer1: &PeerId, peer2: &PeerId, salt: &Salt) -> Distance {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_neighborhood() {
        let local = Local::new();
        let nh = Neighborhood::<2>::new(local);
    }

    #[test]
    fn zero_distance() {}
}
