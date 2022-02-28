// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::vertex::Vertex;

use bee_message::MessageId;

use hashbrown::{hash_map::DefaultHashBuilder, raw::RawTable};
use parking_lot::RwLock;
use rand::Rng;

use std::{
    hash::{BuildHasher, Hash, Hasher},
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};

fn equivalent_id(message_id: &MessageId) -> impl Fn(&(MessageId, Vertex)) -> bool + '_ {
    move |(k, _)| message_id.eq(k)
}

type Table = RwLock<RawTable<(MessageId, Vertex)>>;

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

    fn make_hash(&self, message_id: &MessageId) -> u64 {
        let mut state = self.hash_builder.build_hasher();
        message_id.hash(&mut state);
        state.finish()
    }

    fn make_hasher(&self) -> impl Fn(&(MessageId, Vertex)) -> u64 + '_ {
        move |(message_id, _)| self.make_hash(message_id)
    }

    fn get_table(&self, hash: u64) -> &Table {
        let index = hash as usize % self.tables.len();
        // SAFETY: `index < self.tables.len()` by construction.
        unsafe { self.tables.get_unchecked(index) }
    }

    pub(crate) fn get_map<T>(&self, message_id: &MessageId, f: impl FnOnce(&Vertex) -> T) -> Option<T> {
        let hash = self.make_hash(message_id);

        let guard = self.get_table(hash).read();
        let output = guard.get(hash, equivalent_id(message_id)).map(|(_, v)| f(v));
        drop(guard);

        output
    }

    pub(crate) fn get_and_then<T>(&self, message_id: &MessageId, f: impl FnOnce(&Vertex) -> Option<T>) -> Option<T> {
        let hash = self.make_hash(message_id);

        let guard = self.get_table(hash).read();
        let output = guard.get(hash, equivalent_id(message_id)).and_then(|(_, v)| f(v));
        drop(guard);

        output
    }

    pub(crate) fn get_mut_map<T>(&self, message_id: &MessageId, f: impl FnOnce(&mut Vertex) -> T) -> Option<T> {
        let hash = self.make_hash(message_id);

        let mut guard = self.get_table(hash).write();
        let output = guard.get_mut(hash, equivalent_id(message_id)).map(|(_, v)| f(v));
        drop(guard);

        output
    }

    pub(crate) fn get_mut_and_then<T>(
        &self,
        message_id: &MessageId,
        f: impl FnOnce(&mut Vertex) -> Option<T>,
    ) -> Option<T> {
        let hash = self.make_hash(message_id);

        let mut guard = self.get_table(hash).write();
        let output = guard.get_mut(hash, equivalent_id(message_id)).and_then(|(_, v)| f(v));
        drop(guard);

        output
    }

    pub(crate) fn pop_random(&self, max_retries: usize) -> Option<Vertex> {
        let mut retries = 0;

        while retries < max_retries {
            let index = rand::thread_rng().gen_range(0..self.tables.len());

            // SAFETY: `index < self.tables.len()` by construction.
            if let Some(mut table) = unsafe { self.tables.get_unchecked(index) }.try_write() {
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

    pub(crate) fn get_mut_or_insert_map<T>(&self, message_id: MessageId, f: impl FnOnce(&mut Vertex) -> T) -> T {
        let hash = self.make_hash(&message_id);
        let mut guard = self.get_table(hash).write();

        let bucket = if let Some(bucket) = guard.find(hash, equivalent_id(&message_id)) {
            bucket
        } else {
            let bucket = guard.insert(hash, (message_id, Vertex::empty()), self.make_hasher());
            self.len.fetch_add(1, Ordering::Relaxed);

            bucket
        };
        // SAFETY: We are holding the lock over the table, which means that no other thread could have modified the
        // table to make this bucket invalid.
        let output = f(unsafe { &mut bucket.as_mut().1 });
        drop(guard);
        output
    }

    pub(crate) fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }
}
