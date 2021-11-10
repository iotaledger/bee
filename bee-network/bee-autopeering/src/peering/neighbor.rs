// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    hash,
    local::{salt::Salt, Local},
    peer::{peer_id::PeerId, Peer},
};

use prost::bytes::{Buf, Bytes};

use std::{
    cmp, fmt,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    vec,
};

/// The distance between the local entity and a neighbor.
pub type Distance = u32;

// TODO: revisit dead code
#[allow(dead_code)]
pub(crate) const MAX_DISTANCE: Distance = 4294967295;
pub(crate) const SIZE_INBOUND: usize = 4;
pub(crate) const SIZE_OUTBOUND: usize = 4;

/// Decides whether a peer is a suitable neighbor, or not.
pub trait NeighborValidator
where
    Self: Send + Sync + Clone,
{
    /// Returns `true` if the given [`Peer`](crate::peer::Peer) is a valid neighbor.
    fn is_valid(&self, peer: &Peer) -> bool;
}

// A neighbor is a peer with a (distance) metric.
#[derive(Debug)]
pub(crate) struct Neighbor {
    peer: Peer,
    distance: Distance,
}

impl Neighbor {
    pub(crate) fn new(peer: Peer, distance: Distance) -> Self {
        Self { peer, distance }
    }

    pub(crate) fn peer(&self) -> &Peer {
        &self.peer
    }

    pub(crate) fn distance(&self) -> Distance {
        self.distance
    }

    pub(crate) fn into_peer(self) -> Peer {
        self.peer
    }

    // TODO: revisit dead code
    #[allow(dead_code)]
    pub(crate) fn into_distance(self) -> Distance {
        self.distance
    }
}

impl fmt::Display for Neighbor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.peer().peer_id(), self.distance())
    }
}

impl Eq for Neighbor {}
impl PartialEq for Neighbor {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}
impl PartialOrd for Neighbor {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Neighbor {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.distance.cmp(&other.distance)
    }
}

impl AsRef<Peer> for Neighbor {
    fn as_ref(&self) -> &Peer {
        &self.peer
    }
}

#[derive(Clone, Default)]
pub(crate) struct Neighborhood<const N: usize, const INBOUND: bool> {
    inner: Arc<RwLock<NeighborhoodInner<N, INBOUND>>>,
}

impl<const N: usize, const INBOUND: bool> Neighborhood<N, INBOUND> {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn lock_select(&self, candidate: Neighbor) -> Option<Peer> {
        let mut write = self.write();
        write.select(candidate)
    }

    // TODO: do not expose lock guards, because this leads to dead locks if not very careful.
    pub fn read(&self) -> RwLockReadGuard<NeighborhoodInner<N, INBOUND>> {
        self.inner.read().expect("error getting read access")
    }

    // TODO: do not expose lock guards, because this leads to dead locks if not very careful.
    pub fn write(&self) -> RwLockWriteGuard<NeighborhoodInner<N, INBOUND>> {
        self.inner.write().expect("error getting write access")
    }
}

#[derive(Debug)]
pub(crate) struct NeighborhoodInner<const N: usize, const INBOUND: bool> {
    neighbors: Vec<Neighbor>,
}

impl<const N: usize, const INBOUND: bool> NeighborhoodInner<N, INBOUND> {
    // TODO: revisit dead code
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn insert_neighbor(&mut self, peer: Peer, local: &Local) -> bool {
        // If the peer already exists remove it.
        // NOTE: It's a bit less efficient doing it like this, but the code requires less branching this way.
        let _ = self.remove_neighbor(peer.peer_id());

        if self.neighbors.len() >= N {
            return false;
        }

        // Calculate the distance to that peer.
        let distance = salt_distance(local.read().peer_id(), peer.peer_id(), &{
            if INBOUND {
                local.read().private_salt().expect("missing private salt").clone()
            } else {
                local.read().public_salt().expect("missing public salt").clone()
            }
        });

        self.neighbors.push(Neighbor { distance, peer });

        true
    }

    pub(crate) fn remove_neighbor(&mut self, peer_id: &PeerId) -> Option<Peer> {
        if self.neighbors.is_empty() {
            None
        } else if let Some(index) = self.neighbors.iter().position(|pd| pd.peer().peer_id() == peer_id) {
            let Neighbor { peer, .. } = self.neighbors.remove(index);
            Some(peer)
        } else {
            None
        }
    }

    /// Check whether the candidate is a suitable neighbor.
    pub(crate) fn select(&mut self, candidate: Neighbor) -> Option<Peer> {
        if let Some(furthest) = self.find_furthest() {
            if &candidate < furthest {
                Some(candidate.into_peer())
            } else {
                None
            }
        } else {
            Some(candidate.into_peer())
        }
    }

    /// From the candidate list pick the first, that is closer than the currently furthest neighbor.
    pub(crate) fn select_from_candidate_list<'a>(&mut self, candidates: &'a [&'a Neighbor]) -> Option<&'a Peer> {
        if candidates.is_empty() {
            None
        } else if let Some(furthest) = self.find_furthest() {
            for candidate in candidates {
                if *candidate < furthest {
                    return Some(candidate.peer());
                }
            }
            None
        } else {
            // Any candidate can be selected: pick the first.
            Some(candidates[0].peer())
        }
    }

    pub(crate) fn find_furthest(&mut self) -> Option<&Neighbor> {
        if self.neighbors.len() >= N {
            self.neighbors.sort_unstable();
            self.neighbors.last()
        } else {
            None
        }
    }

    pub(crate) fn remove_furthest(&mut self) -> Option<Peer> {
        // Note: Both methods require unique access to `self`, so we need to clone the peer id.
        if let Some(peer_id) = self.find_furthest().map(|d| *d.peer().peer_id()) {
            self.remove_neighbor(&peer_id)
        } else {
            None
        }
    }

    pub(crate) fn update_distances(&mut self, local: &Local) {
        let local_id = *local.read().peer_id();
        let salt = if INBOUND {
            local.read().private_salt().expect("missing private salt").clone()
        } else {
            local.read().public_salt().expect("missing public salt").clone()
        };

        self.neighbors.iter_mut().for_each(|pd| {
            pd.distance = salt_distance(&local_id, pd.peer().peer_id(), &salt);
        });
    }

    pub(crate) fn is_full(&self) -> bool {
        self.neighbors.len() == N
    }

    // TODO: revisit dead code
    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.neighbors.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.neighbors.len()
    }

    pub(crate) fn clear(&mut self) {
        self.neighbors.clear();
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Peer> {
        self.neighbors.iter().map(|d| d.peer())
    }
}

impl<const N: usize, const INBOUND: bool> Default for NeighborhoodInner<N, INBOUND> {
    fn default() -> Self {
        Self {
            neighbors: Vec::with_capacity(N),
        }
    }
}

impl<const N: usize, const INBOUND: bool> fmt::Display for NeighborhoodInner<N, INBOUND> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.neighbors.len(), N)
    }
}

// hive.go:
// returns the distance (uint32) between x and y by xoring the hash of x and (y + salt):
// xor( hash(x), hash(y+salt) )[:4] as little-endian uint32
pub(crate) fn salt_distance(peer1: &PeerId, peer2: &PeerId, salt: &Salt) -> Distance {
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
        let outbound_nh = Neighborhood::<2, false>::new();
        for i in 0u8..5 {
            outbound_nh.write().insert_neighbor(Peer::new_test_peer(i), &local);
        }
        assert_eq!(outbound_nh.read().len(), 2);
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
        let salted_distance = salt_distance(&peer_id1, &peer_id2, &salt);
        assert_eq!(1184183819, salted_distance);
    }
}
