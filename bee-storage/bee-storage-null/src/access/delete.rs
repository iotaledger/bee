// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Storage;

use bee_storage::access::Delete;

impl<K, V> Delete<K, V> for Storage {
    fn delete(&self, _key: &K) -> Result<(), Self::Error> {
        Ok(())
    }
}
