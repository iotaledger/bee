// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::{Multiaddr, PeerId};
use tokio::sync::RwLock;

use std::{collections::HashSet, hash::Hash, sync::Arc};

const DEFAULT_BANNED_PEER_CAPACITY: usize = 32;
const DEFAULT_BANNED_ADDR_CAPACITY: usize = 16;

pub type PeerBanlist = Banlist<PeerId>;
pub type AddrBanlist = Banlist<Multiaddr>;

#[derive(Clone, Default)]
pub struct Banlist<T: Hash + Eq>(Arc<RwLock<HashSet<T>>>);

impl Banlist<PeerId> {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashSet::with_capacity(
            DEFAULT_BANNED_PEER_CAPACITY,
        ))))
    }
}

impl Banlist<Multiaddr> {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashSet::with_capacity(
            DEFAULT_BANNED_ADDR_CAPACITY,
        ))))
    }
}

impl<T> Banlist<T>
where
    T: Hash + Eq,
{
    pub async fn insert(&self, value: T) -> bool {
        self.0.write().await.insert(value)
    }

    pub async fn contains(&self, value: &T) -> bool {
        self.0.read().await.contains(value)
    }

    pub async fn remove(&self, value: &T) -> bool {
        self.0.write().await.remove(value)
    }
}
