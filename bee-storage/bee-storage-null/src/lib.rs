// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod access;

use bee_storage::backend::StorageBackend;

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

    fn version(&self) -> Result<Option<bee_storage::system::StorageVersion>, Self::Error> {
        Ok(None)
    }

    fn health(&self) -> Result<Option<bee_storage::system::StorageHealth>, Self::Error> {
        Ok(None)
    }

    fn set_health(&self, _health: bee_storage::system::StorageHealth) -> Result<(), Self::Error> {
        Ok(())
    }
}
