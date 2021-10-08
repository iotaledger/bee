// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Delete access operations.

use crate::{column_families::*, Storage};

use bee_message::{Message, MessageId, MessageMetadata};
use bee_storage::{access::Delete, StorageBackend};

impl Delete<MessageId, Message> for Storage {
    fn delete(&self, message_id: &MessageId) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE)?, message_id)?;

        Ok(())
    }
}

impl Delete<MessageId, MessageMetadata> for Storage {
    fn delete(&self, message_id: &MessageId) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_METADATA)?, message_id)?;

        Ok(())
    }
}
