// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Insert access operations.

use crate::{column_families::*, Storage};

use bee_message::{Message, MessageId, MessageMetadata};
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
        .put_cf(storage.cf_handle(CF_SYSTEM)?, [*key], value.pack_to_vec())?;

    Ok(())
}

impl Insert<MessageId, Message> for Storage {
    fn insert(&self, message_id: &MessageId, message: &Message) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE)?,
            message_id,
            message.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Insert<MessageId, MessageMetadata> for Storage {
    fn insert(
        &self,
        message_id: &MessageId,
        message_metadata: &MessageMetadata,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_METADATA)?,
            message_id,
            message_metadata.pack_to_vec(),
        )?;

        Ok(())
    }
}
