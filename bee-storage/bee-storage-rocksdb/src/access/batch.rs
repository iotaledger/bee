// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Batch access operations.

use crate::{column_families::*, Storage};

use bee_message::{Message, MessageId, MessageMetadata};
use bee_packable::Packable;
use bee_storage::{
    access::{Batch, BatchBuilder},
    StorageBackend,
};

use rocksdb::{WriteBatch, WriteOptions};

/// A writing batch that can be applied atomically.
#[derive(Default)]
pub struct StorageBatch {
    inner: WriteBatch,
    // TODO uncomment when needed
    // key_buf: Vec<u8>,
    value_buf: Vec<u8>,
}

impl BatchBuilder for Storage {
    type Batch = StorageBatch;

    fn batch_commit(&self, batch: Self::Batch, durability: bool) -> Result<(), <Self as StorageBackend>::Error> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(!durability);
        self.inner.write_opt(batch.inner, &write_options)?;

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
            .put_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE)?, message_id, &batch.value_buf);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        message_id: &MessageId,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch
            .inner
            .delete_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE)?, message_id);

        Ok(())
    }
}

impl Batch<MessageId, MessageMetadata> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        message_id: &MessageId,
        message_metadata: &MessageMetadata,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        message_metadata.pack(&mut batch.value_buf).unwrap();

        batch.inner.put_cf(
            self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_METADATA)?,
            message_id,
            &batch.value_buf,
        );

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        message_id: &MessageId,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch
            .inner
            .delete_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_METADATA)?, message_id);

        Ok(())
    }
}
