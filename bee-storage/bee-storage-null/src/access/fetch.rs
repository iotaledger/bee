// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Storage;

use bee_storage::access::Fetch;

impl<K, V> Fetch<K, V> for Storage {
    fn fetch(&self, _key: &K) -> Result<Option<V>, Self::Error> {
        Ok(None)
    }
}
