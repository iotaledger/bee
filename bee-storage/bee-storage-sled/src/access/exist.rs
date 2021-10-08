// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Exist access operations.

use crate::{trees::*, Storage};

use bee_message::{Message, MessageId, MessageMetadata};
use bee_storage::{access::Exist, StorageBackend};

impl Exist<MessageId, Message> for Storage {
    fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MESSAGE_ID_TO_MESSAGE)?
            .contains_key(message_id)?)
    }
}

impl Exist<MessageId, MessageMetadata> for Storage {
    fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MESSAGE_ID_TO_MESSAGE_METADATA)?
            .contains_key(message_id)?)
    }
}
