// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    hash::{BuildHasher, Hash, Hasher},
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};

use bee_block::BlockId;
use hashbrown::{hash_map::DefaultHashBuilder, raw::RawTable};
use rand::Rng;
use tokio::sync::{RwLock, RwLockMappedWriteGuard, RwLockReadGuard, RwLockWriteGuard};

use crate::vertex::Vertex;

fn equivalent_id(message_id: &BlockId) -> impl Fn(&(BlockId, Vertex)) -> bool + '_ {
    move |(k, _)| message_id.eq(k)
}

type Table = RwLock<RawTable<(BlockId, Vertex)>>;

pub(crate) struct Vertices {
    hash_builder: DefaultHashBuilder,
    tables: Box<[Table]>,
    len: AtomicUsize,
}

impl Vertices {
    pub(crate) fn new(num_partitions: NonZeroUsize) -> Self {
        Self {
            hash_builder: DefaultHashBuilder::default(),
            tables: (0..num_partitions.into())
                .map(|_| RwLock::new(RawTable::default()))
                .collect(),
            len: AtomicUsize::default(),
        }
    }

    fn make_hash(&self, message_id: &BlockId) -> u64 {
        let mut state = self.hash_builder.build_hasher();
        message_id.hash(&mut state);
        state.finish()
    }

    fn make_hasher(&self) -> impl Fn(&(BlockId, Vertex)) -> u64 + '_ {
        move |(message_id, _)| self.make_hash(message_id)
    }

    fn get_table(&self, hash: u64) -> &Table {
        let index = hash as usize % self.tables.len();
        // SAFETY: `index < self.tables.len()` by construction.
        unsafe { self.tables.get_unchecked(index) }
    }

    pub(crate) async fn get(&self, message_id: &BlockId) -> Option<RwLockReadGuard<'_, Vertex>> {
        let hash = self.make_hash(message_id);
        let table = self.get_table(hash).read().await;

        RwLockReadGuard::try_map(table, |table| match table.get(hash, equivalent_id(message_id)) {
            Some((_, v)) => Some(v),
            None => None,
        })
        .ok()
    }

    pub(crate) async fn get_mut(&self, message_id: &BlockId) -> Option<RwLockMappedWriteGuard<'_, Vertex>> {
        let hash = self.make_hash(message_id);
        let table = self.get_table(hash).write().await;

        RwLockWriteGuard::try_map(table, |table| match table.get_mut(hash, equivalent_id(message_id)) {
            Some((_, v)) => Some(v),
            None => None,
        })
        .ok()
    }

    pub(crate) async fn pop_random(&self, max_retries: usize) -> Option<Vertex> {
        let mut retries = 0;

        while retries < max_retries {
            let index = rand::thread_rng().gen_range(0..self.tables.len());

            // SAFETY: `index < self.tables.len()` by construction.
            if let Ok(mut table) = unsafe { self.tables.get_unchecked(index) }.try_write() {
                // SAFETY: We are holding the lock over the table, which means that no other thread could have modified,
                // added nor deleted any bucket. This applies to all the following `unsafe` blocks.
                let buckets = unsafe { table.iter() };

                for bucket in buckets {
                    let (_, vertex) = unsafe { bucket.as_ref() };

                    if vertex.can_evict() {
                        self.len.fetch_sub(1, Ordering::Relaxed);
                        let (_, vertex) = unsafe { table.remove(bucket) };

                        return Some(vertex);
                    }
                }
            }

            retries += 1;
            log::trace!(
                "Retrying cache eviction for table index {} (attempt #{}).",
                index,
                retries
            );
        }

        None
    }

    pub(crate) async fn get_mut_or_empty(&self, message_id: BlockId) -> RwLockMappedWriteGuard<'_, Vertex> {
        let hash = self.make_hash(&message_id);
        let table = self.get_table(hash).write().await;

        RwLockWriteGuard::map(table, |table| {
            let bucket = if let Some(bucket) = table.find(hash, equivalent_id(&message_id)) {
                bucket
            } else {
                let bucket = table.insert(hash, (message_id, Vertex::empty()), self.make_hasher());
                self.len.fetch_add(1, Ordering::Relaxed);

                bucket
            };
            // SAFETY: We are holding the lock over the table, which means that no other thread could have modified the
            // table to make this bucket invalid.
            unsafe { &mut bucket.as_mut().1 }
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }
}
