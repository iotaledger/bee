// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Storage;

use bee_storage::access::Truncate;

impl<K, V> Truncate<K, V> for Storage {
    fn truncate(&self) -> Result<(), Self::Error> {
        Ok(())
    }
}
