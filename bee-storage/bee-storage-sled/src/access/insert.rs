// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Insert access operations.

use crate::{trees::*, Storage};

use bee_message::{Message, MessageId};
use bee_packable::packable::Packable;
use bee_storage::{access::Insert, system::System, StorageBackend};

impl Insert<u8, System> for Storage {
    fn insert(&self, key: &u8, value: &System) -> Result<(), <Self as StorageBackend>::Error> {
        // Packing to bytes can't fail.
        self.inner.insert(&[*key], value.pack_to_vec().unwrap())?;

        Ok(())
    }
}

impl Insert<MessageId, Message> for Storage {
    fn insert(&self, message_id: &MessageId, message: &Message) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_MESSAGE_ID_TO_MESSAGE)?
            // Packing to bytes can't fail.
            .insert(message_id, message.pack_to_vec().unwrap())?;

        Ok(())
    }
}
