// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Update access operations.

use bee_block::BlockId;
use bee_storage::{access::Update, backend::StorageBackend};
use bee_tangle::block_metadata::BlockMetadata;

use crate::storage::Storage;

macro_rules! impl_update {
    ($key:ty, $value:ty, $field:ident) => {
        impl Update<$key, $value> for Storage {
            fn update(&self, k: &$key, f: impl FnMut(&mut $value)) -> Result<(), <Self as StorageBackend>::Error> {
                Ok(self.inner.write()?.$field.update(k, f))
            }
        }
    };
}

impl_update!(BlockId, BlockMetadata, block_id_to_metadata);
