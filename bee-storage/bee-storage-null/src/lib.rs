// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_storage::{
    access::{Fetch, Insert},
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

impl<T, U> Insert<T, U> for Storage {
    fn insert(&self, _key: &T, _value: &U) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<T, U> Fetch<T, U> for Storage {
    fn fetch(&self, _key: &T) -> Result<Option<U>, Self::Error> {
        Ok(None)
    }
}
