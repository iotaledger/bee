// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Storage;

use bee_storage::access::Update;

impl<K, V> Update<K, V> for Storage {
    fn update(&self, _key: &K, _f: impl FnMut(&mut V)) -> Result<(), Self::Error> {
        Ok(())
    }
}
