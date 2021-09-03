// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Exist access operations.

use crate::{storage::Storage, trees::*};

use bee_message::{Message, MessageId};
use bee_storage::{access::Exist, backend::StorageBackend};

impl Exist<MessageId, Message> for Storage {
    fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MESSAGE_ID_TO_MESSAGE)?
            .contains_key(message_id)?)
    }
}
