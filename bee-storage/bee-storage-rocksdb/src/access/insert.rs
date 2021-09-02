// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
};

use bee_message::{Message, MessageId};
use bee_packable::Packable;
use bee_storage::{access::Insert, system::System};

impl Insert<u8, System> for Storage {
    fn insert(&self, key: &u8, value: &System) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            // Packing to bytes can't fail.
            .put_cf(self.cf_handle(CF_SYSTEM)?, [*key], value.pack_to_vec().unwrap())?;

        Ok(())
    }
}

impl Insert<MessageId, Message> for Storage {
    fn insert(&self, message_id: &MessageId, message: &Message) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE)?,
            message_id,
            // Packing to bytes can't fail.
            message.pack_to_vec().unwrap(),
        )?;

        Ok(())
    }
}
