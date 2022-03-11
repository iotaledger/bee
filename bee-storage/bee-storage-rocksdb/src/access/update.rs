// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{column_families::*, storage::Storage};

use bee_common::packable::Packable;
use bee_message::MessageId;
use bee_storage::access::Update;
use bee_tangle::metadata::MessageMetadata;

impl Update<MessageId, MessageMetadata> for Storage {
    fn update(&self, message_id: &MessageId, mut f: impl FnMut(&mut MessageMetadata)) -> Result<(), Self::Error> {
        let cf_handle = self.cf_handle(CF_MESSAGE_ID_TO_METADATA)?;

        let guard = self.locks.message_id_to_metadata.write();

        if let Some(v) = self.inner.get_cf(cf_handle, message_id)? {
            // Unpacking from storage is fine.
            let mut metadata = MessageMetadata::unpack_unchecked(&mut v.as_slice()).unwrap();

            f(&mut metadata);

            self.inner.put_cf(cf_handle, message_id, metadata.pack_new())?;
        }

        drop(guard);

        Ok(())
    }
}
