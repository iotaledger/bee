// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_storage::access::Delete;

use crate::Storage;

impl<K, V> Delete<K, V> for Storage {
    fn delete_op(&self, _key: &K) -> Result<(), Self::Error> {
        Ok(())
    }
}
