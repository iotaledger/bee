// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::storage::Error;

use std::{
    collections::{hash_map::IntoIter as HashMapIter, HashMap},
    hash::Hash,
    iter::Map,
    option::IntoIter as OptionIter,
    sync::RwLock,
    vec::IntoIter as VecIter,
};

pub(crate) type TableIter<K, V> = Map<HashMapIter<K, V>, fn((K, V)) -> Result<(K, V), Error>>;
pub(crate) type TableMultiFetchIter<V> = Map<VecIter<Option<V>>, fn(Option<V>) -> Result<Option<V>, Error>>;

pub(crate) struct Table<K, V> {
    inner: RwLock<HashMap<K, V>>,
}

impl<K, V> Default for Table<K, V> {
    fn default() -> Self {
        Self {
            inner: RwLock::new(Default::default()),
        }
    }
}

impl<K: Hash + Eq + Clone, V: Clone> Table<K, V> {
    pub(crate) fn fetch(&self, k: &K) -> Result<Option<V>, Error> {
        Ok(self.inner.read()?.get(k).cloned())
    }

    pub(crate) fn exist(&self, k: &K) -> Result<bool, Error> {
        Ok(self.inner.read()?.contains_key(k))
    }

    pub(crate) fn insert(&self, k: &K, v: &V) -> Result<(), Error> {
        self.inner.write()?.insert(k.clone(), v.clone());

        Ok(())
    }

    pub(crate) fn delete(&self, k: &K) -> Result<(), Error> {
        self.inner.write()?.remove(k);

        Ok(())
    }

    pub(crate) fn truncate(&self) -> Result<(), Error> {
        self.inner.write()?.clear();

        Ok(())
    }

    pub(crate) fn iter(&self) -> Result<TableIter<K, V>, Error> {
        Ok(self.inner.read()?.clone().into_iter().map(Ok))
    }

    pub(crate) fn batch_commit(&self, batch: TableBatch<K, V>) -> Result<(), Error> {
        let mut inner = self.inner.write()?;

        for op in batch.0 {
            match op {
                BatchOp::Insert(k, v) => inner.insert(k, v),
                BatchOp::Delete(k) => inner.remove(&k),
            };
        }

        Ok(())
    }

    pub(crate) fn multi_fetch(&self, ks: &[K]) -> Result<TableMultiFetchIter<V>, Error> {
        let inner = self.inner.read()?;
        let mut vs = Vec::with_capacity(ks.len());

        for k in ks {
            let v = inner.get(k).cloned();
            vs.push(v);
        }

        Ok(vs.into_iter().map(Ok))
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
    inner: RwLock<HashMap<K, Vec<V>>>,
}

impl<K, V> Default for VecTable<K, V> {
    fn default() -> Self {
        Self {
            inner: RwLock::new(Default::default()),
        }
    }
}

impl<K: Hash + Eq + Clone, V: Clone + Eq> VecTable<K, V> {
    pub(crate) fn fetch(&self, k: &K) -> Result<Option<Vec<V>>, Error> {
        Ok(self.inner.read()?.get(k).cloned().or_else(|| Some(vec![])))
    }

    pub(crate) fn exist(&self, (k, v): &(K, V)) -> Result<bool, Error> {
        Ok(self.inner.read()?.get(k).map_or(false, |vs| vs.contains(v)))
    }

    pub(crate) fn insert(&self, (k, v): &(K, V), _: &()) -> Result<(), Error> {
        let mut inner = self.inner.write()?;

        let vs = inner.entry(k.clone()).or_default();

        if !vs.contains(v) {
            vs.push(v.clone());
        }

        Ok(())
    }

    pub(crate) fn delete(&self, (k, v): &(K, V)) -> Result<(), Error> {
        if let Some(vs) = self.inner.write()?.get_mut(k) {
            for (i, found) in vs.iter().enumerate() {
                if found == v {
                    vs.remove(i);
                    break;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn truncate(&self) -> Result<(), Error> {
        self.inner.write()?.clear();

        Ok(())
    }

    pub(crate) fn iter(&self) -> Result<VecTableIter<K, V>, Error> {
        Ok(VecTableIter::new(self.inner.read()?.clone().into_iter()))
    }

    pub(crate) fn batch_commit(&self, batch: TableBatch<(K, V), ()>) -> Result<(), Error> {
        let mut inner = self.inner.write()?;

        for op in batch.0 {
            match op {
                BatchOp::Insert((k, v), ()) => {
                    let vs = inner.entry(k).or_default();

                    if !vs.contains(&v) {
                        vs.push(v);
                    }
                }
                BatchOp::Delete((k, v)) => {
                    if let Some(vs) = inner.get_mut(&k) {
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

        Ok(())
    }
}

pub(crate) struct VecBinTable<K, V> {
    inner: RwLock<HashMap<K, Vec<V>>>,
}

impl<K, V> Default for VecBinTable<K, V> {
    fn default() -> Self {
        Self {
            inner: RwLock::new(Default::default()),
        }
    }
}

impl<K: Hash + Eq + Clone, V: Clone + Eq + Ord> VecBinTable<K, V> {
    pub(crate) fn fetch(&self, k: &K) -> Result<Option<Vec<V>>, Error> {
        Ok(self.inner.read()?.get(k).cloned().or_else(|| Some(vec![])))
    }

    pub(crate) fn exist(&self, (k, v): &(K, V)) -> Result<bool, Error> {
        Ok(self
            .inner
            .read()?
            .get(k)
            .map_or(false, |vs| vs.binary_search(v).is_ok()))
    }

    pub(crate) fn insert(&self, (k, v): &(K, V), _: &()) -> Result<(), Error> {
        let mut inner = self.inner.write()?;

        let vs = inner.entry(k.clone()).or_default();

        if let Err(i) = vs.binary_search(v) {
            vs.insert(i, v.clone());
        }

        Ok(())
    }

    pub(crate) fn delete(&self, (k, v): &(K, V)) -> Result<(), Error> {
        if let Some(vs) = self.inner.write()?.get_mut(k) {
            if let Ok(i) = vs.binary_search(v) {
                vs.remove(i);
            }
        }

        Ok(())
    }

    pub(crate) fn truncate(&self) -> Result<(), Error> {
        self.inner.write()?.clear();

        Ok(())
    }

    pub(crate) fn iter(&self) -> Result<VecTableIter<K, V>, Error> {
        Ok(VecTableIter::new(self.inner.read()?.clone().into_iter()))
    }

    pub(crate) fn batch_commit(&self, batch: TableBatch<(K, V), ()>) -> Result<(), Error> {
        let mut inner = self.inner.write()?;

        for op in batch.0 {
            match op {
                BatchOp::Insert((k, v), ()) => {
                    let vs = inner.entry(k).or_default();

                    if let Err(i) = vs.binary_search(&v) {
                        vs.insert(i, v);
                    }
                }
                BatchOp::Delete((k, v)) => {
                    if let Some(vs) = inner.get_mut(&k) {
                        if let Ok(i) = vs.binary_search(&v) {
                            vs.remove(i);
                        }
                    }
                }
            };
        }

        Ok(())
    }
}

pub(crate) type SingletonTableIter<V> = Map<OptionIter<V>, fn(V) -> Result<((), V), Error>>;

pub(crate) struct SingletonTable<V> {
    inner: RwLock<Option<V>>,
}

impl<V> Default for SingletonTable<V> {
    fn default() -> Self {
        Self {
            inner: RwLock::new(Default::default()),
        }
    }
}

impl<V: Clone> SingletonTable<V> {
    pub(crate) fn fetch(&self, _: &()) -> Result<Option<V>, Error> {
        Ok(self.inner.read()?.clone())
    }

    pub(crate) fn exist(&self, _: &()) -> Result<bool, Error> {
        Ok(self.inner.read()?.is_some())
    }

    pub(crate) fn insert(&self, _: &(), v: &V) -> Result<(), Error> {
        *self.inner.write()? = Some(v.clone());

        Ok(())
    }

    pub(crate) fn delete(&self, _: &()) -> Result<(), Error> {
        *self.inner.write()? = None;

        Ok(())
    }

    pub(crate) fn truncate(&self) -> Result<(), Error> {
        *self.inner.write()? = None;

        Ok(())
    }

    pub(crate) fn iter(&self) -> Result<SingletonTableIter<V>, Error> {
        Ok(self.inner.read()?.clone().into_iter().map(|v| Ok(((), v))))
    }

    pub(crate) fn batch_commit(&self, batch: TableBatch<(), V>) -> Result<(), Error> {
        let mut inner = self.inner.write()?;

        for op in batch.0 {
            *inner = match op {
                BatchOp::Insert((), v) => Some(v),
                BatchOp::Delete(()) => None,
            };
        }

        Ok(())
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
