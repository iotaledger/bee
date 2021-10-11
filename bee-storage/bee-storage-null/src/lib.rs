// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod access;

use bee_storage::{
    access::{Delete, Exist, Fetch, Insert, Truncate},
    backend::StorageBackend,
};

pub struct Storage;

impl StorageBackend for Storage {
    type ConfigBuilder = ();
    type Config = ();
    type Error = std::convert::Infallible;

    fn start(_config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Storage)
    }

    fn shutdown(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn size(&self) -> Result<Option<usize>, Self::Error> {
        Ok(None)
    }

    fn get_health(&self) -> Result<Option<bee_storage::system::StorageHealth>, Self::Error> {
        Ok(None)
    }

    fn set_health(&self, _health: bee_storage::system::StorageHealth) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<K, V> Delete<K, V> for Storage {
    fn delete(&self, _key: &K) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<K, V> Exist<K, V> for Storage {
    fn exist(&self, _key: &K) -> Result<bool, Self::Error> {
        Ok(false)
    }
}

impl<K, V> Fetch<K, V> for Storage {
    fn fetch(&self, _key: &K) -> Result<Option<V>, Self::Error> {
        Ok(None)
    }
}

impl<K, V> Insert<K, V> for Storage {
    fn insert(&self, _key: &K, _value: &V) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<K, V> Truncate<K, V> for Storage {
    fn truncate(&self) -> Result<(), Self::Error> {
        Ok(())
    }
}
