// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Insert access operations.

use crate::{storage::Storage, trees::*};

use bee_packable::packable::Packable;

use bee_message::{Message, MessageId};
use bee_storage::{access::Insert, backend::StorageBackend, system::System};

impl Insert<u8, System> for Storage {
    fn insert(&self, key: &u8, value: &System) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.insert(&[*key], Packable::pack_to_vec(value).unwrap())?;

        Ok(())
    }
}

impl Insert<MessageId, Message> for Storage {
    fn insert(&self, message_id: &MessageId, message: &Message) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_MESSAGE_ID_TO_MESSAGE)?
            .insert(message_id, Packable::pack_to_vec(message).unwrap())?;

        Ok(())
    }
}
