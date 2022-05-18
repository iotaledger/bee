// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_storage::access::{Insert, InsertStrict};

use crate::Storage;

impl<K, V> Insert<K, V> for Storage {
    fn insert_op(&self, _key: &K, _value: &V) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<K, V> InsertStrict<K, V> for Storage {
    fn insert_strict_op(&self, _key: &K, _value: &V) -> Result<(), Self::Error> {
        Ok(())
    }
}
