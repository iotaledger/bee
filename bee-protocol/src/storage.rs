// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{payload::indexation::HashedIndex, Message, MessageId};
use bee_storage::{
    access::{AsStream, Fetch, Insert},
    backend,
};
use bee_tangle::{
    metadata::MessageMetadata,
    milestone::{Milestone, MilestoneIndex},
    solid_entry_point::SolidEntryPoint,
};

pub trait StorageBackend:
    backend::StorageBackend
    + Insert<MessageId, Message>
    + Insert<MessageId, MessageMetadata>
    + Insert<(MessageId, MessageId), ()>
    + Insert<MilestoneIndex, Milestone>
    + Insert<(HashedIndex, MessageId), ()>
    + Fetch<MessageId, Message>
    + Fetch<MessageId, MessageMetadata>
    + Fetch<MessageId, Vec<MessageId>>
    + Fetch<MilestoneIndex, Milestone>
    + for<'a> AsStream<'a, SolidEntryPoint, MilestoneIndex>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Insert<MessageId, Message>
        + Insert<MessageId, MessageMetadata>
        + Insert<(MessageId, MessageId), ()>
        + Insert<MilestoneIndex, Milestone>
        + Insert<(HashedIndex, MessageId), ()>
        + Fetch<MessageId, Message>
        + Fetch<MessageId, MessageMetadata>
        + Fetch<MessageId, Vec<MessageId>>
        + Fetch<MessageId, Vec<MessageId>>
        + Fetch<MilestoneIndex, Milestone>
        + for<'a> AsStream<'a, SolidEntryPoint, MilestoneIndex>
{
}
