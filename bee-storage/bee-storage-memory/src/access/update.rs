// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Update access operations.

use bee_message::MessageId;
use bee_storage::{access::Update, backend::StorageBackend};
use bee_tangle::message_metadata::MessageMetadata;

use crate::storage::Storage;

macro_rules! impl_update {
    ($key:ty, $value:ty, $field:ident) => {
        impl Update<$key, $value> for Storage {
            fn update(&self, k: &$key, f: impl FnMut(&mut $value)) -> Result<(), <Self as StorageBackend>::Error> {
                Ok(self.inner.write()?.$field.update(k, f))
            }
        }
    };
}

impl_update!(MessageId, MessageMetadata, message_id_to_metadata);
