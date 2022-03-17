// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{metadata::MessageMetadata, solid_entry_point::SolidEntryPoint};

use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    Message, MessageId,
};

use bee_storage::{
    access::{Exist, Fetch, Insert, InsertStrict, Update},
    backend,
};

/// A blanket-implemented helper trait for the storage layer.
pub trait StorageBackend:
    backend::StorageBackend
    + Insert<MessageId, Message>
    + Insert<(MessageId, MessageId), ()>
    + Insert<MilestoneIndex, Milestone>
    + Insert<SolidEntryPoint, MilestoneIndex>
    + InsertStrict<MessageId, MessageMetadata>
    + Exist<MessageId, Message>
    + Exist<MilestoneIndex, Milestone>
    + Fetch<MessageId, Message>
    + Fetch<MessageId, MessageMetadata>
    + Fetch<MessageId, Vec<MessageId>>
    + Fetch<MilestoneIndex, Milestone>
    + Update<MessageId, MessageMetadata>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Insert<MessageId, Message>
        + Insert<(MessageId, MessageId), ()>
        + Insert<MilestoneIndex, Milestone>
        + Insert<SolidEntryPoint, MilestoneIndex>
        + InsertStrict<MessageId, MessageMetadata>
        + Exist<MessageId, Message>
        + Exist<MilestoneIndex, Milestone>
        + Fetch<MessageId, Message>
        + Fetch<MessageId, MessageMetadata>
        + Fetch<MessageId, Vec<MessageId>>
        + Fetch<MilestoneIndex, Milestone>
        + Update<MessageId, MessageMetadata>
{
}
