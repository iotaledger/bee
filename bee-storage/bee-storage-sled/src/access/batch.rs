// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Batch access operations.

use crate::{storage::Storage, trees::*};

use bee_message::{Message, MessageId};
use bee_packable::packable::Packable;
use bee_storage::{
    access::{Batch, BatchBuilder},
    backend::StorageBackend,
};

use std::collections::HashMap;

/// A writing batch that can be applied atomically.
#[derive(Default)]
pub struct StorageBatch {
    inner: HashMap<&'static str, sled::Batch>,
    // TODO uncomment when needed
    // key_buf: Vec<u8>,
    value_buf: Vec<u8>,
}

impl BatchBuilder for Storage {
    type Batch = StorageBatch;

    fn batch_commit(&self, batch: Self::Batch, _durability: bool) -> Result<(), <Self as StorageBackend>::Error> {
        for (tree, batch) in batch.inner {
            self.inner.open_tree(tree)?.apply_batch(batch)?;
        }

        Ok(())
    }
}

impl Batch<MessageId, Message> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        message_id: &MessageId,
        message: &Message,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        message.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .entry(TREE_MESSAGE_ID_TO_MESSAGE)
            .or_default()
            .insert(message_id.as_ref(), batch.value_buf.as_slice());

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        message_id: &MessageId,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch
            .inner
            .entry(TREE_MESSAGE_ID_TO_MESSAGE)
            .or_default()
            .remove(message_id.as_ref());

        Ok(())
    }
}
