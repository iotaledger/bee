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

// A neighbor is a peer with a distance metric.
#[derive(Clone, Debug)]
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
    /// Creates a new empty neighborhood.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Inserts a peer to the neighborhood.
    pub(crate) fn insert_neighbor(&self, peer: Peer, local: &Local) -> bool {
        self.write().insert_neighbor(peer, local)
    }

    /// Removes a peer from the neighborhood.
    pub(crate) fn remove_neighbor(&self, peer_id: &PeerId) -> Option<Peer> {
        self.write().remove_neighbor(peer_id)
    }

    /// Checks whether the candidate is a preferred neighbor.
    pub(crate) fn is_preferred(&self, candidate: &Neighbor) -> bool {
        self.write().is_preferred(candidate)
    }

    /// Picks the first candidate that is closer than the currently furthest neighbor.
    pub(crate) fn select_from_candidate_list<'a>(&self, candidates: &'a [&'a Neighbor]) -> Option<&'a Peer> {
        self.write().select_from_candidate_list(candidates)
    }

    /// Removes the furthest neighbor from the neighborhood.
    pub(crate) fn remove_furthest_if_full(&self) -> Option<Peer> {
        self.write().remove_furthest_if_full()
    }

    /// Updates all distances to the neighbors (e.g. after a salt update).
    pub(crate) fn update_distances(&self, local: &Local) {
        self.write().update_distances(local);
    }

    /// Clears the neighborhood, removing all neighbors.
    pub(crate) fn clear(&self) {
        self.write().clear();
    }

    /// Returns the number of neighbors within the neighborhood.
    pub(crate) fn len(&self) -> usize {
        self.read().len()
    }

    /// Returns whether the neighborhood is full, i.e. the upper bound is reached.
    pub(crate) fn is_full(&self) -> bool {
        self.read().is_full()
    }

    /// Collect all peers belonging to the neighborhood into a `Vec`.
    pub(crate) fn peers(&self) -> Vec<Peer> {
        self.read().neighbors.iter().map(|d| d.peer()).cloned().collect()
    }

    fn read(&self) -> RwLockReadGuard<NeighborhoodInner<N, INBOUND>> {
        self.inner.read().expect("error getting read access")
    }

    fn write(&self) -> RwLockWriteGuard<NeighborhoodInner<N, INBOUND>> {
        self.inner.write().expect("error getting write access")
    }
}

#[derive(Debug)]
pub(crate) struct NeighborhoodInner<const N: usize, const INBOUND: bool> {
    neighbors: Vec<Neighbor>,
}

impl<const N: usize, const INBOUND: bool> NeighborhoodInner<N, INBOUND> {
    fn insert_neighbor(&mut self, peer: Peer, local: &Local) -> bool {
        // If the peer already exists remove it.
        // NOTE: It's a bit less efficient doing it like this, but the code requires less branching this way.
        let _ = self.remove_neighbor(peer.peer_id());

        if self.neighbors.len() >= N {
            return false;
        }

        // Calculate the distance to that peer.
        let distance = salt_distance(&local.peer_id(), peer.peer_id(), &{
            if INBOUND {
                local.private_salt()
            } else {
                local.public_salt()
            }
        });

        self.neighbors.push(Neighbor { distance, peer });

        true
    }

    fn remove_neighbor(&mut self, peer_id: &PeerId) -> Option<Peer> {
        if self.neighbors.is_empty() {
            None
        } else if let Some(index) = self.neighbors.iter().position(|pd| pd.peer().peer_id() == peer_id) {
            let Neighbor { peer, .. } = self.neighbors.remove(index);
            Some(peer)
        } else {
            None
        }
    }

    fn is_preferred(&mut self, candidate: &Neighbor) -> bool {
        if let Some(furthest) = self.find_furthest_if_full() {
            candidate < furthest
        } else {
            true
        }
    }

    fn select_from_candidate_list<'a>(&mut self, candidates: &'a [&'a Neighbor]) -> Option<&'a Peer> {
        if candidates.is_empty() {
            None
        } else if let Some(furthest) = self.find_furthest_if_full() {
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

    fn find_furthest_if_full(&mut self) -> Option<&Neighbor> {
        if self.neighbors.len() >= N {
            self.neighbors.sort_unstable();
            self.neighbors.last()
        } else {
            None
        }
    }

    fn remove_furthest_if_full(&mut self) -> Option<Peer> {
        // Note: Both methods require unique access to `self`, so we need to copy the peer id.
        if let Some(peer_id) = self.find_furthest_if_full().map(|d| *d.peer().peer_id()) {
            self.remove_neighbor(&peer_id)
        } else {
            None
        }
    }

    fn update_distances(&mut self, local: &Local) {
        let local_id = local.peer_id();
        let salt = if INBOUND {
            local.private_salt()
        } else {
            local.public_salt()
        };

        self.neighbors.iter_mut().for_each(|pd| {
            pd.distance = salt_distance(&local_id, pd.peer().peer_id(), &salt);
        });
    }

    fn len(&self) -> usize {
        self.neighbors.len()
    }

    fn is_full(&self) -> bool {
        self.neighbors.len() == N
    }

    fn clear(&mut self) {
        self.neighbors.clear();
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
    let hash1 = hash::data_hash(peer1.id_bytes());
    let hash2 = hash::data_hash(&concat(peer2.id_bytes(), salt.bytes()));

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
        let hash1 = hash::data_hash(peer1.id_bytes());
        let hash2 = hash::data_hash(peer2.id_bytes());
        let xored = xor(hash1, hash2);
        Bytes::copy_from_slice(&xored[..4]).get_u32_le()
    }

    #[test]
    fn neighborhood_size_limit() {
        let local = Local::generate();
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
