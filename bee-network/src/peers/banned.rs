// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::PeerId;

use dashmap::DashSet;

use std::{hash::Hash, sync::Arc};

const DEFAULT_BANNED_PEER_CAPACITY: usize = 64;
const DEFAULT_BANNED_ADDR_CAPACITY: usize = 32;

pub type BannedPeerList = BannedList<PeerId>;
pub type BannedAddrList = BannedList<String>;

#[derive(Clone, Default)]
pub struct BannedList<T: Hash + Eq>(Arc<DashSet<T>>);

impl BannedList<PeerId> {
    pub fn new() -> Self {
        Self(Arc::new(DashSet::with_capacity(DEFAULT_BANNED_PEER_CAPACITY)))
    }
}

impl BannedList<String> {
    pub fn new() -> Self {
        Self(Arc::new(DashSet::with_capacity(DEFAULT_BANNED_ADDR_CAPACITY)))
    }
}

impl<T> BannedList<T>
where
    T: Hash + Eq,
{
    pub fn insert(&self, value: T) -> bool {
        self.0.insert(value)
    }

    pub fn contains(&self, value: &T) -> bool {
        self.0.contains(value)
    }

    pub fn remove(&self, value: &T) -> bool {
        self.0.remove(value).is_some()
    }

    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.0.len()
    }
}
