// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::metadata::MessageMetadata;

use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    solid_entry_point::SolidEntryPoint,
    Message, MessageId,
};
use bee_snapshot::storage::StorageBackend as SnapshotStorageBackend;
use bee_storage::{
    access::{Fetch, Insert},
    backend,
};

pub trait StorageBackend:
    backend::StorageBackend
    + Insert<MessageId, Message>
    + Insert<MessageId, MessageMetadata>
    + Insert<(MessageId, MessageId), ()>
    + Insert<MilestoneIndex, Milestone>
    + Insert<SolidEntryPoint, MilestoneIndex>
    + Fetch<MessageId, Message>
    + Fetch<MessageId, MessageMetadata>
    + Fetch<MessageId, Vec<MessageId>>
    + Fetch<MilestoneIndex, Milestone>
    + SnapshotStorageBackend
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Insert<MessageId, Message>
        + Insert<MessageId, MessageMetadata>
        + Insert<(MessageId, MessageId), ()>
        + Insert<MilestoneIndex, Milestone>
        + Insert<SolidEntryPoint, MilestoneIndex>
        + Fetch<MessageId, Message>
        + Fetch<MessageId, MessageMetadata>
        + Fetch<MessageId, Vec<MessageId>>
        + Fetch<MilestoneIndex, Milestone>
        + SnapshotStorageBackend
{
}
