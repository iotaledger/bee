// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::identity::PeerId;

use std::collections::{vec_deque, VecDeque};

#[derive(Clone)]
pub(crate) struct PeerRing<P, const N: usize>(VecDeque<P>);

impl<P: AsRef<PeerId>, const N: usize> PeerRing<P, N> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    // Returns `false`, if the list already contains the id, otherwise `true`.
    pub(crate) fn append(&mut self, item: P) -> bool {
        if self.contains(item.as_ref()) {
            false
        } else {
            if self.is_full() {
                self.remove_oldest();
            }
            self.0.push_front(item);
            true
        }
    }

    pub(crate) fn remove_oldest(&mut self) -> Option<P> {
        self.0.pop_back()
    }

    pub(crate) fn remove(&mut self, peer_id: &PeerId) -> Option<P> {
        if let Some(index) = self.find_index(peer_id) {
            self.remove_at(index)
        } else {
            None
        }
    }

    pub(crate) fn remove_at(&mut self, index: usize) -> Option<P> {
        self.0.remove(index)
    }

    pub(crate) fn contains(&self, peer_id: &PeerId) -> bool {
        self.0.iter().any(|v| v.as_ref() == peer_id)
    }

    pub(crate) fn find_index(&self, peer_id: &PeerId) -> Option<usize> {
        self.0.iter().position(|v| v.as_ref() == peer_id)
    }

    pub(crate) fn get(&self, index: usize) -> Option<&P> {
        self.0.get(index)
    }

    pub(crate) fn newest(&self) -> Option<&P> {
        self.0.get(0)
    }

    pub(crate) fn oldest(&self) -> Option<&P> {
        self.0.get(self.0.len() - 1)
    }

    pub(crate) fn rotate_backwards(&mut self) {
        self.0.rotate_left(1);
    }

    pub(crate) fn rotate_forwards(&mut self) {
        self.0.rotate_right(1);
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn is_full(&self) -> bool {
        self.len() >= N
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // TODO: mark as 'const fn' once stable.
    // Compiler error hints to issue #57563 <https://github.com/rust-lang/rust/issues/57563>.
    pub(crate) fn max_size(&self) -> usize {
        N
    }

    pub(crate) fn iter(&self) -> vec_deque::Iter<P> {
        self.0.iter()
    }
}

impl<P, const N: usize> Default for PeerRing<P, N> {
    fn default() -> Self {
        Self(VecDeque::with_capacity(N))
    }
}
