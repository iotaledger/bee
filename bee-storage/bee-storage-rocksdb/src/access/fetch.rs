// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Fetch access operations.

use crate::{column_families::*, Storage};

use bee_message::{Message, MessageId, MessageMetadata};
use bee_packable::Packable;
use bee_storage::{access::Fetch, system::System, StorageBackend};

impl Fetch<u8, System> for Storage {
    fn fetch(&self, key: &u8) -> Result<Option<System>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_SYSTEM)?, [*key])?
            // Unpacking from storage slice can't fail.
            .map(|v| System::unpack_unchecked(&mut v.as_slice()).unwrap()))
    }
}

impl Fetch<MessageId, Message> for Storage {
    fn fetch(&self, message_id: &MessageId) -> Result<Option<Message>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE)?, message_id)?
            // Unpacking from storage slice can't fail.
            .map(|v| Message::unpack_unchecked(&mut v.as_slice()).unwrap()))
    }
}

impl Fetch<MessageId, MessageMetadata> for Storage {
    fn fetch(&self, message_id: &MessageId) -> Result<Option<MessageMetadata>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_METADATA)?, message_id)?
            // Unpacking from storage slice can't fail.
            .map(|v| MessageMetadata::unpack_unchecked(&mut v.as_slice()).unwrap()))
    }
}
