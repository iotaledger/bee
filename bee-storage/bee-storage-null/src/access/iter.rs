// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::marker::PhantomData;

use bee_storage::{access::AsIterator, backend::StorageBackend};

use crate::Storage;

pub struct StorageIterator<K, V> {
    marker: PhantomData<(K, V)>,
}

impl<K, V> StorageIterator<K, V> {
    fn new() -> Self {
        Self { marker: PhantomData }
    }
}

impl<K, V> Iterator for StorageIterator<K, V> {
    type Item = Result<(K, V), <Storage as StorageBackend>::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<'a, K, V> AsIterator<'a, K, V> for Storage {
    type AsIter = StorageIterator<K, V>;

    fn iter_op(&'a self) -> Result<Self::AsIter, Self::Error> {
        Ok(StorageIterator::new())
    }
}
