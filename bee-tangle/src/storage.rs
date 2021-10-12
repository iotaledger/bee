// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageId, MessageMetadata};
use bee_storage::{
    access::{Fetch, Insert},
    backend,
};

/// A blanket-implemented helper trait with all tangle storage requirements.
pub trait StorageBackend:
    backend::StorageBackend
    + Fetch<MessageId, Message>
    + Fetch<MessageId, MessageMetadata>
    + Insert<MessageId, Message>
    + Insert<MessageId, MessageMetadata>
{
}

impl<S> StorageBackend for S where
    S: backend::StorageBackend
        + Fetch<MessageId, Message>
        + Fetch<MessageId, MessageMetadata>
        + Insert<MessageId, Message>
        + Insert<MessageId, MessageMetadata>
{
}
