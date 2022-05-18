// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_storage::access::{Batch, BatchBuilder};

use crate::Storage;

#[derive(Default)]
pub struct StorageBatch;

impl BatchBuilder for Storage {
    type Batch = StorageBatch;

    fn batch_commit(&self, _batch: Self::Batch, _durability: bool) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<K, V> Batch<K, V> for Storage {
    fn batch_insert_op(&self, _batch: &mut Self::Batch, _key: &K, _value: &V) -> Result<(), Self::Error> {
        Ok(())
    }

    fn batch_delete_op(&self, _batch: &mut Self::Batch, _key: &K) -> Result<(), Self::Error> {
        Ok(())
    }
}
