// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Fetch access operations.

use crate::{trees::*, Storage};

use bee_message::{Message, MessageId, MessageMetadata};
use bee_packable::PackableExt;
use bee_storage::{access::Fetch, system::System, StorageBackend};

impl Fetch<u8, System> for Storage {
    fn fetch(&self, &key: &u8) -> Result<Option<System>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get(&[key])?
            // Unpacking from storage slice can't fail.
            .map(|v| System::unpack_unverified(&mut v.as_ref()).unwrap()))
    }
}

impl Fetch<MessageId, Message> for Storage {
    fn fetch(&self, message_id: &MessageId) -> Result<Option<Message>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MESSAGE_ID_TO_MESSAGE)?
            .get(message_id)?
            // Unpacking from storage slice can't fail.
            .map(|v| Message::unpack_unverified(&mut v.as_ref()).unwrap()))
    }
}

impl Fetch<MessageId, MessageMetadata> for Storage {
    fn fetch(&self, message_id: &MessageId) -> Result<Option<MessageMetadata>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MESSAGE_ID_TO_MESSAGE_METADATA)?
            .get(message_id)?
            // Unpacking from storage slice can't fail.
            .map(|v| MessageMetadata::unpack_unverified(&mut v.as_ref()).unwrap()))
    }
}
