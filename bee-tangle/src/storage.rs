// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageId, MessageMetadata};
use bee_storage::{
    access::{Fetch, Insert},
    backend,
};

///
pub trait StorageBackend:
    backend::StorageBackend
    + Fetch<MessageId, Message>
    + Fetch<MessageId, MessageMetadata>
    + Insert<MessageId, Message>
    + Insert<MessageId, MessageMetadata>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Fetch<MessageId, Message>
        + Fetch<MessageId, MessageMetadata>
        + Insert<MessageId, Message>
        + Insert<MessageId, MessageMetadata>
{
}
