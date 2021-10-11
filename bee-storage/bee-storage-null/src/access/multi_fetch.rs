// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Storage;

use bee_storage::{access::MultiFetch, backend::StorageBackend};

use std::marker::PhantomData;

pub struct MultiIter<K, V> {
    marker: PhantomData<(K, V)>,
}

impl<K, V> MultiIter<K, V> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { marker: PhantomData }
    }
}

impl<K, V> Iterator for MultiIter<K, V> {
    type Item = Result<Option<V>, <Storage as StorageBackend>::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<'a, K: 'a, V: 'a> MultiFetch<'a, K, V> for Storage {
    type Iter = MultiIter<K, V>;

    fn multi_fetch(&'a self, _keys: &'a [K]) -> Result<Self::Iter, Self::Error> {
        Ok(MultiIter::new())
    }
}
