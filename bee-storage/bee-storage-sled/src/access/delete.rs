// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Delete access operations.

use crate::{storage::Storage, trees::*};

use bee_message::{Message, MessageId};
use bee_storage::{access::Delete, backend::StorageBackend};

impl Delete<MessageId, Message> for Storage {
    fn delete(&self, message_id: &MessageId) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.open_tree(TREE_MESSAGE_ID_TO_MESSAGE)?.remove(message_id)?;

        Ok(())
    }
}
