// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    Message, MessageId,
};
use bee_storage::{
    access::{Fetch},
    backend,
};

pub trait StorageBackend:
    backend::StorageBackend
    + Fetch<MessageId, Message>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Fetch<MessageId, Message>
{
}