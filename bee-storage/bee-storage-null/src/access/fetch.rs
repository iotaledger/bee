// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_storage::access::Fetch;

use crate::Storage;

impl<K, V> Fetch<K, V> for Storage {
    fn fetch_op(&self, _key: &K) -> Result<Option<V>, Self::Error> {
        Ok(None)
    }
}
