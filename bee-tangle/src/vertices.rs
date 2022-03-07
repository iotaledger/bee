// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::vertex::Vertex;

use bee_message::MessageId;

use hashbrown::{
    hash_map::DefaultHashBuilder,
    raw::{Bucket, RawTable},
};
use rand::Rng;
use tokio::sync::{RwLock, RwLockMappedWriteGuard, RwLockReadGuard, RwLockWriteGuard};

use std::{
    hash::{BuildHasher, Hash, Hasher},
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};

fn equivalent_id(message_id: &MessageId) -> impl Fn(&(MessageId, Vertex)) -> bool + '_ {
    move |(k, _)| message_id.eq(k)
}

type Table = RwLock<RawTable<(MessageId, Vertex)>>;

pub(crate) struct OccupiedEntry<'a> {
    // hash: u64,
    // message_id: Option<MessageId>,
    elem: Bucket<(MessageId, Vertex)>,
    table: RwLockWriteGuard<'a, RawTable<(MessageId, Vertex)>>,
}

impl<'a> OccupiedEntry<'a> {
    pub(crate) fn into_mut(self) -> RwLockMappedWriteGuard<'a, Vertex> {
        RwLockWriteGuard::map(self.table, |_| unsafe { &mut self.elem.as_mut().1 })
    }
}

pub(crate) struct VacantEntry<'a> {
    hash: u64,
    message_id: MessageId,
    table: RwLockWriteGuard<'a, RawTable<(MessageId, Vertex)>>,
    hash_builder: &'a DefaultHashBuilder,
    len: &'a AtomicUsize,
}

impl<'a> VacantEntry<'a> {
    pub(crate) fn insert_empty(self) -> RwLockMappedWriteGuard<'a, Vertex> {
        RwLockWriteGuard::map(self.table, |table| {
            let entry = table.insert_entry(self.hash, (self.message_id, Vertex::empty()), move |(message_id, _)| {
                make_hash(&self.hash_builder, message_id)
            });
            self.len.fetch_add(1, Ordering::Relaxed);
            &mut entry.1
        })
    }
}

pub(crate) enum Entry<'a> {
    Occupied(OccupiedEntry<'a>),
    Vacant(VacantEntry<'a>),
}

impl<'a> Entry<'a> {
    pub(crate) fn or_empty(self) -> RwLockMappedWriteGuard<'a, Vertex> {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert_empty(),
        }
    }
}

fn make_hash(hash_builder: &DefaultHashBuilder, message_id: &MessageId) -> u64 {
    let mut state = hash_builder.build_hasher();
    message_id.hash(&mut state);
    state.finish()
}

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

    fn get_table(&self, hash: u64) -> &Table {
        let index = hash as usize % self.tables.len();
        // SAFETY: `index < self.tables.len()` by construction.
        unsafe { self.tables.get_unchecked(index) }
    }

    pub(crate) async fn get(&self, message_id: &MessageId) -> Option<RwLockReadGuard<'_, Vertex>> {
        let hash = make_hash(&self.hash_builder, message_id);
        let table = self.get_table(hash).read().await;

        RwLockReadGuard::try_map(table, |table| match table.get(hash, equivalent_id(message_id)) {
            Some((_, v)) => Some(v),
            None => None,
        })
        .ok()
    }

    pub(crate) async fn get_mut(&self, message_id: &MessageId) -> Option<RwLockMappedWriteGuard<'_, Vertex>> {
        let hash = make_hash(&self.hash_builder, message_id);
        let table = self.get_table(hash).write().await;

        RwLockWriteGuard::try_map(table, |table| match table.get_mut(hash, equivalent_id(message_id)) {
            Some((_, v)) => Some(v),
            None => None,
        })
        .ok()
    }

    pub(crate) fn pop_random(&self, max_retries: usize) -> Option<Vertex> {
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

    pub(crate) async fn entry(&self, message_id: MessageId) -> Entry<'_> {
        let hash = make_hash(&self.hash_builder, &message_id);
        let table = self.get_table(hash).write().await;

        if let Some(elem) = table.find(hash, equivalent_id(&message_id)) {
            Entry::Occupied(OccupiedEntry {
                // hash,
                // message_id: Some(message_id),
                elem,
                table,
            })
        } else {
            Entry::Vacant(VacantEntry {
                hash,
                message_id,
                table,
                hash_builder: &self.hash_builder,
                len: &self.len,
            })
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    pub(crate) async fn real_len(&self) -> usize {
        let mut len = 0;
        for table in self.tables.iter() {
            len += table.read().await.len();
        }
        len
    }
}
