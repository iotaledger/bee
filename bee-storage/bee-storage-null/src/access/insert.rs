// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Storage;

use bee_storage::access::{Insert, InsertStrict};

impl<K, V> Insert<K, V> for Storage {
    fn insert(&self, _key: &K, _value: &V) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<K, V> InsertStrict<K, V> for Storage {
    fn insert_strict(&self, _key: &K, _value: &V) -> Result<(), Self::Error> {
        Ok(())
    }
}
