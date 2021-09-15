// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Insert access operations.

use crate::{column_families::*, Storage};

use bee_message::{Message, MessageId};
use bee_packable::Packable;
use bee_storage::{access::Insert, system::System, StorageBackend};

/// This would normally be `impl Insert<u8, System> for Storage` but there is no way to have a private trait impl and
/// only the [`Storage`] itself should be able to insert system values.
pub(crate) fn insert_u8_system(
    storage: &Storage,
    key: &u8,
    value: &System,
) -> Result<(), <Storage as StorageBackend>::Error> {
    storage
        .inner
        // Packing to bytes can't fail.
        .put_cf(storage.cf_handle(CF_SYSTEM)?, [*key], value.pack_to_vec().unwrap())?;

    Ok(())
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
