// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashSet,
    hash::Hash,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

pub enum Direction {
    Inbound,
    Outbound,
}

pub type ConnectedList = FilterList<SocketAddr>;

#[derive(Clone)]
pub struct FilterList<T: Eq + Hash>(Arc<RwLock<HashSet<T>>>);

impl<T: Eq + Hash> FilterList<T> {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashSet::new())))
    }

    pub fn add(&self, item: T) {
        self.0.write().expect("error getting the lock").insert(item);
    }

    // TODO: update this list on connection loss
    #[allow(dead_code)]
    pub fn remove(&self, item: T) -> bool {
        self.0.write().expect("error getting the lock").remove(&item)
    }

    pub fn contains(&self, item: T) -> bool {
        self.0.read().expect("error getting the lock").contains(&item)
    }
}
