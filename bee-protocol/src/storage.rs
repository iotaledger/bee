// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::tangle::MessageMetadata;

use bee_message::{Message, MessageId};
use bee_storage::{
    access::{Batch, BatchBuilder, Delete, Exist, Fetch, Insert},
    storage,
};

pub trait Backend: storage::Backend + Insert<MessageId, Message> + Insert<MessageId, MessageMetadata> {}

impl<T> Backend for T where T: storage::Backend + Insert<MessageId, Message> + Insert<MessageId, MessageMetadata> {}
