// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Truncate access operations.

use crate::{trees::*, Storage};

use bee_message::{Message, MessageId, MessageMetadata};
use bee_storage::{access::Truncate, StorageBackend};

macro_rules! impl_truncate {
    ($key:ty, $value:ty, $cf:expr) => {
        impl Truncate<$key, $value> for Storage {
            fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
                self.inner.drop_tree($cf)?;

                Ok(())
            }
        }
    };
}

impl_truncate!(MessageId, Message, TREE_MESSAGE_ID_TO_MESSAGE);
impl_truncate!(MessageId, MessageMetadata, TREE_MESSAGE_ID_TO_MESSAGE_METADATA);
