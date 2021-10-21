// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{hash, identity::PeerId, local::Local, peer::Peer, salt::Salt};

use prost::bytes::{Buf, Bytes};

use std::{
    cmp,
    collections::{BTreeMap, BTreeSet},
    fmt, vec,
};

pub(crate) type Distance = u32;

pub(crate) const MAX_DISTANCE: Distance = 4294967295;
pub(crate) const SIZE_INBOUND: usize = 4;
pub(crate) const SIZE_OUTBOUND: usize = 4;

#[derive(Debug)]
pub(crate) struct PeerDistance {
    pub(crate) peer: Peer,
    pub(crate) distance: Distance,
}

impl PeerDistance {
    pub(crate) fn new(peer: Peer, distance: Distance) -> Self {
        Self { peer, distance }
    }

    pub(crate) fn peer(&self) -> &Peer {
        &self.peer
    }

    pub(crate) fn distance(&self) -> Distance {
        self.distance
    }
}

impl fmt::Display for PeerDistance {
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

impl AsRef<Peer> for PeerDistance {
    fn as_ref(&self) -> &Peer {
        &self.peer
    }
}

#[derive(Debug, Default)]
pub(crate) struct Neighborhood<const N: usize, const INBOUND: bool> {
    local: Local,
    neighbors: Vec<PeerDistance>,
}

impl<const N: usize, const INBOUND: bool> Neighborhood<N, INBOUND> {
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
        let distance = if INBOUND {
            salted_distance(
                &self.local.peer_id(),
                &peer.peer_id(),
                &self.local.private_salt().expect("missing private salt"),
            )
        } else {
            salted_distance(
                &self.local.peer_id(),
                &peer.peer_id(),
                &self.local.public_salt().expect("missing public salt"),
            )
        };

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

        let salt = if INBOUND {
            self.local.private_salt().expect("missing private salt")
        } else {
            self.local.public_salt().expect("missing public salt")
        };

        self.neighbors.iter_mut().for_each(|pd| {
            pd.distance = salted_distance(&local_peer_id, &pd.peer().peer_id(), &salt);
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

    pub(crate) fn clear(&mut self) {
        self.neighbors.clear();
    }
}

impl<const N: usize, const INBOUND: bool> fmt::Display for Neighborhood<N, INBOUND> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.neighbors.len(), N)
    }
}

// Allows us to iterate the peers in the neighborhood with a for-loop.
impl<'a, const N: usize, const INBOUND: bool> IntoIterator for &'a Neighborhood<N, INBOUND> {
    type Item = Peer;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.neighbors
            .iter()
            // FIXME: can we prevent the clone?
            .map(|pd| pd.peer().clone())
            .collect::<Vec<_>>()
            .into_iter()
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
    let l: usize = N + M;
    let mut bytes = vec![0u8; l];
    bytes[0..N].copy_from_slice(bytes1);
    bytes[N..l].copy_from_slice(bytes2);
    bytes
}

fn xor<const N: usize>(a: [u8; N], b: [u8; N]) -> [u8; N] {
    let mut xored = [0u8; N];
    // TODO: use array_zip when available (rust-lang/rust#80094)
    a.iter()
        .zip(b.iter())
        .enumerate()
        .for_each(|(i, (a, b))| xored[i] = a ^ b);

    xored
}

#[cfg(test)]
mod tests {
    use crypto::signatures::ed25519::SecretKey as PrivateKey;

    use super::*;

    fn distance(peer1: &PeerId, peer2: &PeerId) -> Distance {
        let hash1 = hash::sha256(peer1.id_bytes());
        let hash2 = hash::sha256(peer2.id_bytes());
        let xored = xor(hash1, hash2);
        Bytes::copy_from_slice(&xored[..4]).get_u32_le()
    }

    #[test]
    fn neighborhood_size_limit() {
        let local = Local::new();
        let mut outbound_nh = Neighborhood::<2, false>::new(local);
        for i in 0u8..5 {
            outbound_nh.insert_peer(Peer::new_test_peer(i));
        }
        assert_eq!(outbound_nh.num_neighbors(), 2);
    }

    #[test]
    fn byte_array_concatenation() {
        let bytes1 = [1, 2, 3, 4];
        let bytes2 = [5, 6, 7, 8, 9];
        assert_eq!(vec![1, 2, 3, 4, 5, 6, 7, 8, 9], concat(&bytes1, &bytes2));
    }

    #[test]
    fn distance_calculation() {
        let peer_id1 = PeerId::new_static();
        let peer_id2 = PeerId::new_static();
        assert_eq!(peer_id1, peer_id2);

        let distance = distance(&peer_id1, &peer_id2);
        assert_eq!(0, distance);

        let salt = Salt::new_zero_salt();
        let salted_distance = salted_distance(&peer_id1, &peer_id2, &salt);
        assert_eq!(1184183819, salted_distance);
    }
}
