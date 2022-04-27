// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Insert access operations.

use bee_message::MessageId;
use bee_storage::access::Update;
use bee_tangle::metadata::MessageMetadata;
use packable::PackableExt;

use crate::{storage::Storage, trees::*};

impl Update<MessageId, MessageMetadata> for Storage {
    fn update(&self, message_id: &MessageId, mut f: impl FnMut(&mut MessageMetadata)) -> Result<(), Self::Error> {
        self.inner
            .open_tree(TREE_MESSAGE_ID_TO_METADATA)?
            .fetch_and_update(message_id, move |opt_bytes| {
                opt_bytes.map(|mut bytes| {
                    // Unpacking from storage is fine.
                    let mut metadata = MessageMetadata::unpack_unverified(&mut bytes).unwrap();
                    f(&mut metadata);
                    metadata.pack_to_vec()
                })
            })?;

        Ok(())
    }
}
