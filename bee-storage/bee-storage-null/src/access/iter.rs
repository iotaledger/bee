use crate::Storage;

use bee_storage::{access::AsIterator, backend::StorageBackend};

use std::marker::PhantomData;

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

    fn iter(&'a self) -> Result<Self::AsIter, Self::Error> {
        Ok(StorageIterator::new())
    }
}
