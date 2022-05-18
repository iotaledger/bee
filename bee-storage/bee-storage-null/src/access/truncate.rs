// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_storage::access::Truncate;

use crate::Storage;

impl<K, V> Truncate<K, V> for Storage {
    fn truncate_op(&self) -> Result<(), Self::Error> {
        Ok(())
    }
}
