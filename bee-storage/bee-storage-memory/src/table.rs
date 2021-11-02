// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::storage::Error;

use std::{
    collections::{hash_map::IntoIter as HashMapIter, HashMap},
    hash::Hash,
    iter::Map,
    option::IntoIter as OptionIter,
    vec::IntoIter as VecIter,
};

pub(crate) type TableIter<K, V> = Map<HashMapIter<K, V>, fn((K, V)) -> Result<(K, V), Error>>;
pub(crate) type TableMultiFetchIter<V> = Map<VecIter<Option<V>>, fn(Option<V>) -> Result<Option<V>, Error>>;

pub(crate) struct Table<K, V> {
    inner: HashMap<K, V>,
}

impl<K, V> Default for Table<K, V> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K: Hash + Eq + Clone, V: Clone> Table<K, V> {
    pub(crate) fn fetch(&self, k: &K) -> Option<V> {
        self.inner.get(k).cloned()
    }

    pub(crate) fn exist(&self, k: &K) -> bool {
        self.inner.contains_key(k)
    }

    pub(crate) fn insert(&mut self, k: &K, v: &V) {
        self.inner.insert(k.clone(), v.clone());
    }

    pub(crate) fn delete(&mut self, k: &K) {
        self.inner.remove(k);
    }

    pub(crate) fn truncate(&mut self) {
        self.inner.clear();
    }

    pub(crate) fn iter(&self) -> TableIter<K, V> {
        self.inner.clone().into_iter().map(Ok)
    }

    pub(crate) fn batch_commit(&mut self, batch: TableBatch<K, V>) {
        for op in batch.0 {
            match op {
                BatchOp::Insert(k, v) => self.inner.insert(k, v),
                BatchOp::Delete(k) => self.inner.remove(&k),
            };
        }
    }

    pub(crate) fn multi_fetch(&self, ks: &[K]) -> TableMultiFetchIter<V> {
        let mut vs = Vec::with_capacity(ks.len());

        for k in ks {
            let v = self.inner.get(k).cloned();
            vs.push(v);
        }

        vs.into_iter().map(Ok)
    }
}

/// An iterator over the elements of a `VecTable` or `VecBinTable`.
pub struct VecTableIter<K, V> {
    head: Option<(K, Vec<V>)>,
    tail: HashMapIter<K, Vec<V>>,
}

impl<K, V> VecTableIter<K, V> {
    fn new(mut iter: HashMapIter<K, Vec<V>>) -> Self {
        Self {
            head: iter.next(),
            tail: iter,
        }
    }
}

impl<K: Clone, V> Iterator for VecTableIter<K, V> {
    type Item = Result<((K, V), ()), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((k, vs)) = &mut self.head {
            if let Some(v) = vs.pop() {
                return Some(Ok(((k.clone(), v), ())));
            }
        }

        let new_head = self.tail.next()?;
        self.head = Some(new_head);
        self.next()
    }
}

pub(crate) struct VecTable<K, V> {
    inner: HashMap<K, Vec<V>>,
}

impl<K, V> Default for VecTable<K, V> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K: Hash + Eq + Clone, V: Clone + Eq> VecTable<K, V> {
    pub(crate) fn fetch(&self, k: &K) -> Option<Vec<V>> {
        self.inner.get(k).cloned().or_else(|| Some(vec![]))
    }

    pub(crate) fn exist(&self, (k, v): &(K, V)) -> bool {
        self.inner.get(k).map_or(false, |vs| vs.contains(v))
    }

    pub(crate) fn insert(&mut self, (k, v): &(K, V), _: &()) {
        let vs = self.inner.entry(k.clone()).or_default();

        if !vs.contains(v) {
            vs.push(v.clone());
        }
    }

    pub(crate) fn delete(&mut self, (k, v): &(K, V)) {
        if let Some(vs) = self.inner.get_mut(k) {
            for (i, found) in vs.iter().enumerate() {
                if found == v {
                    vs.remove(i);
                    break;
                }
            }
        }
    }

    pub(crate) fn truncate(&mut self) {
        self.inner.clear();
    }

    pub(crate) fn iter(&self) -> VecTableIter<K, V> {
        VecTableIter::new(self.inner.clone().into_iter())
    }

    pub(crate) fn batch_commit(&mut self, batch: TableBatch<(K, V), ()>) {
        for op in batch.0 {
            match op {
                BatchOp::Insert((k, v), ()) => {
                    let vs = self.inner.entry(k).or_default();

                    if !vs.contains(&v) {
                        vs.push(v);
                    }
                }
                BatchOp::Delete((k, v)) => {
                    if let Some(vs) = self.inner.get_mut(&k) {
                        for (i, found) in vs.iter().enumerate() {
                            if found == &v {
                                vs.remove(i);
                                break;
                            }
                        }
                    }
                }
            };
        }
    }
}

pub(crate) struct VecBinTable<K, V> {
    inner: HashMap<K, Vec<V>>,
}

impl<K, V> Default for VecBinTable<K, V> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<K: Hash + Eq + Clone, V: Clone + Eq + Ord> VecBinTable<K, V> {
    pub(crate) fn fetch(&self, k: &K) -> Option<Vec<V>> {
        self.inner.get(k).cloned().or_else(|| Some(vec![]))
    }

    pub(crate) fn exist(&self, (k, v): &(K, V)) -> bool {
        self.inner.get(k).map_or(false, |vs| vs.binary_search(v).is_ok())
    }

    pub(crate) fn insert(&mut self, (k, v): &(K, V), _: &()) {
        let vs = self.inner.entry(k.clone()).or_default();

        if let Err(i) = vs.binary_search(v) {
            vs.insert(i, v.clone());
        }
    }

    pub(crate) fn delete(&mut self, (k, v): &(K, V)) {
        if let Some(vs) = self.inner.get_mut(k) {
            if let Ok(i) = vs.binary_search(v) {
                vs.remove(i);
            }
        }
    }

    pub(crate) fn truncate(&mut self) {
        self.inner.clear();
    }

    pub(crate) fn iter(&self) -> VecTableIter<K, V> {
        VecTableIter::new(self.inner.clone().into_iter())
    }

    pub(crate) fn batch_commit(&mut self, batch: TableBatch<(K, V), ()>) {
        for op in batch.0 {
            match op {
                BatchOp::Insert((k, v), ()) => {
                    let vs = self.inner.entry(k).or_default();

                    if let Err(i) = vs.binary_search(&v) {
                        vs.insert(i, v);
                    }
                }
                BatchOp::Delete((k, v)) => {
                    if let Some(vs) = self.inner.get_mut(&k) {
                        if let Ok(i) = vs.binary_search(&v) {
                            vs.remove(i);
                        }
                    }
                }
            };
        }
    }
}

pub(crate) type SingletonTableIter<V> = Map<OptionIter<V>, fn(V) -> Result<((), V), Error>>;

pub(crate) struct SingletonTable<V> {
    inner: Option<V>,
}

impl<V> Default for SingletonTable<V> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<V: Clone> SingletonTable<V> {
    pub(crate) fn fetch(&self, _: &()) -> Option<V> {
        self.inner.clone()
    }

    pub(crate) fn exist(&self, _: &()) -> bool {
        self.inner.is_some()
    }

    pub(crate) fn insert(&mut self, _: &(), v: &V) {
        self.inner = Some(v.clone());
    }

    pub(crate) fn delete(&mut self, _: &()) {
        self.inner = None;
    }

    pub(crate) fn truncate(&mut self) {
        self.inner = None;
    }

    pub(crate) fn iter(&self) -> SingletonTableIter<V> {
        self.inner.clone().into_iter().map(|v| Ok(((), v)))
    }

    pub(crate) fn batch_commit(&mut self, batch: TableBatch<(), V>) {
        for op in batch.0 {
            self.inner = match op {
                BatchOp::Insert((), v) => Some(v),
                BatchOp::Delete(()) => None,
            };
        }
    }
}

pub(crate) struct TableBatch<K, V>(Vec<BatchOp<K, V>>);

impl<K, V> Default for TableBatch<K, V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K: Clone, V: Clone> TableBatch<K, V> {
    pub(crate) fn insert(&mut self, k: &K, v: &V) {
        self.0.push(BatchOp::Insert(k.clone(), v.clone()));
    }

    pub(crate) fn delete(&mut self, k: &K) {
        self.0.push(BatchOp::Delete(k.clone()));
    }
}

pub(crate) enum BatchOp<K, V> {
    Insert(K, V),
    Delete(K),
}
