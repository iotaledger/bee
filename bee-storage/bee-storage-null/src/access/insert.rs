// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_storage::access::Insert;

use crate::Storage;

impl<K, V> Insert<K, V> for Storage {
    fn insert(&self, _key: &K, _value: &V) -> Result<(), Self::Error> {
        Ok(())
    }
}
