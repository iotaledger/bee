// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Storage;

use bee_storage::access::Exist;

impl<K, V> Exist<K, V> for Storage {
    fn exist(&self, _key: &K) -> Result<bool, Self::Error> {
        Ok(false)
    }
}
