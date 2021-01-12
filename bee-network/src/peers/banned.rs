// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Multiaddr, PeerId};

use tokio::sync::RwLock;

use std::{collections::HashSet, hash::Hash, sync::Arc};

const DEFAULT_BANNED_PEER_CAPACITY: usize = 64;
const DEFAULT_BANNED_ADDR_CAPACITY: usize = 32;

pub type BannedPeerList = BannedList<PeerId>;
pub type BannedAddrList = BannedList<Multiaddr>;

#[derive(Clone, Default)]
pub struct BannedList<T: Hash + Eq>(Arc<RwLock<HashSet<T>>>);

impl BannedList<PeerId> {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashSet::with_capacity(
            DEFAULT_BANNED_PEER_CAPACITY,
        ))))
    }
}

impl BannedList<Multiaddr> {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashSet::with_capacity(
            DEFAULT_BANNED_ADDR_CAPACITY,
        ))))
    }
}

impl<T> BannedList<T>
where
    T: Hash + Eq,
{
    // NOTE: Instead of consuming the value when the insert fails, we return ownership,
    // so the caller can pass it to a handler dealing with that situation.
    pub async fn insert(&self, value: T) -> Option<T> {
        if !self.contains(&value).await {
            self.0.write().await.insert(value);
            None
        } else {
            Some(value)
        }
    }

    pub async fn contains(&self, value: &T) -> bool {
        self.0.read().await.contains(value)
    }

    pub async fn remove(&self, value: &T) -> bool {
        self.0.write().await.remove(value)
    }

    #[allow(dead_code)]
    pub async fn len(&self) -> usize {
        self.0.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn inserted_items_are_unique() {
        let item_list: BannedList<i32> = BannedList(Arc::new(RwLock::new(HashSet::new())));

        item_list.insert(0).await;
        item_list.insert(1).await;
        item_list.insert(1).await;

        assert!(item_list.contains(&0).await);
        assert!(item_list.contains(&1).await);
        assert_eq!(2, item_list.len().await);

        assert!(!item_list.contains(&2).await);
    }

    #[tokio::test]
    async fn inserts_and_removals_update_count() {
        let item_list: BannedList<i32> = BannedList(Arc::new(RwLock::new(HashSet::new())));

        item_list.insert(0).await;
        assert!(item_list.contains(&0).await);
        assert_eq!(1, item_list.len().await);

        item_list.remove(&0).await;
        assert!(!item_list.contains(&0).await);
        assert_eq!(0, item_list.len().await);
    }
}
